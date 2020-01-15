use chashmap::CHashMap;
use dawn_model::{channel::message::Message, id::ChannelId};
use dawn_gateway::shard::event::Event;
use futures_channel::mpsc::{UnboundedSender, unbounded};
use futures_util::stream::{Stream, StreamExt};

use std::{marker::Unpin, sync::Arc, time::Instant};

use crate::wait::{WaitFor, WaitForMultiple};

/// Listens for [`MessageCreate`] events and will remove any messages that match the predicate from the stream.
#[derive(Clone, Default)]
pub struct Listener {
    pub(crate) items: Arc<CHashMap<ChannelId, Vec<ListenerItem>>>,
}
impl Listener {
    /// Handles a Stream of events, removing any [`MessageCreate`] events that match the predicate.
    pub fn handle<'a, S: Stream<Item=Event> + Unpin + Send + 'a>(&'a self, events: S) -> impl Stream<Item=Event> + Unpin + Send + 'a {
        events.filter_map(move |event| {
            let items_map = self.items.clone();
            async move {
                match event {
                    Event::MessageCreate(message) => match matches(items_map, &message) {
                        Some(_) => None,
                        None => Some(Event::MessageCreate(message)),
                    }
                    event => Some(event),
                }
            }
        }).boxed()
    }

    /// Waits for a single [`Message`] that meets the predicate and is in the channel provided.
    ///
    /// This will wait *forever* until a [`Message`] is found or the [`WaitFor`] struct is dropped.
    ///
    /// You may want to timeout the future by using a function like `tokio::timer::timeout` or equivalent.
    ///
    /// [`WaitFor`]: struct.WaitFor.html
    pub fn wait_for<F: Fn(&Message) -> bool + Send + Sync + 'static>(&self, channel: ChannelId, predicate: F) -> WaitFor {
        WaitFor::new(self.wait_for_multiple(channel, Some(1), predicate))
    }

    /// Waits for multiple [`Message`]s that meet the predicate and is in the channel provided.
    ///
    /// This will wait *forever* until a [`Message`] is found `num_messages` times or the [`WaitForMultiple`] struct is dropped.
    ///
    /// You may want to timeout the future by using a function like `tokio::timer::timeout` or equivalent.
    ///
    /// [`WaitForMultiple`]: struct.WaitForMultiple.html
    pub fn wait_for_multiple<F: Fn(&Message) -> bool + Send + Sync + 'static>(&self, channel: ChannelId, num_messages: Option<u8>, predicate: F) -> WaitForMultiple {
        let (sender, receiver) = unbounded();
        let created = Instant::now();
        let item = ListenerItem {
            created,
            num_uses: num_messages,
            predicate: Box::new(predicate),
            sender,
        };
        if let Some(mut items) = self.items.get_mut(&channel) {
            items.push(item);
        } else {
            self.items.insert_new(channel, vec![item]);
        }
        WaitForMultiple { channel_id: channel, created, items_map: self.items.clone(), receiver, num: num_messages }
    }
}

pub(crate) struct ListenerItem {
    /// The number of times this item should be used.
    ///
    /// `None` if it can be used an unlimited amount of times.
    num_uses: Option<u8>,
    /// The condition needed for the message to be used.
    predicate: Box<dyn Fn(&Message) -> bool + Send + Sync>,
    /// The sending end of the channel.
    pub(crate) sender: UnboundedSender<Message>,
    pub(crate) created: Instant,
}

fn matches(items_map: Arc<CHashMap<ChannelId, Vec<ListenerItem>>>, message: &Message) -> Option<()> {
    let mut items = items_map.get_mut(&message.channel_id)?;
    let item = items.iter_mut().find(|item| (item.predicate)(&message) && item.num_uses != Some(0))?;
    item.sender.unbounded_send(message.clone()).ok()?;
    if let Some(ref mut uses) = item.num_uses {
        *uses -= 1;
    }
    Some(())
}
