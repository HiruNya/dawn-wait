use chashmap::CHashMap;
use dawn_model::{channel::message::Message, gateway::payload::MessageCreate, id::ChannelId};
use dawn_gateway::shard::event::Event;
use futures_channel::mpsc::{UnboundedSender, unbounded};
use futures_util::stream::{Stream, StreamExt};

use std::{marker::Unpin, sync::Arc, time::Instant};

use crate::wait::{WaitFor, WaitForMultiple};

/// Listens for `MessageCreate` events and will remove any messages that match the predicate from the stream.
#[derive(Clone, Default)]
pub struct Listener {
    items: Arc<CHashMap<ChannelId, Vec<ListenerItem>>>,
}
impl Listener {
    /// Handles a Stream of events, removing any MessageCreate events that match the predicate.
    pub fn handle<'a, S: Stream<Item=Event> + Unpin + Send + 'a>(&'a self, events: S) -> impl Stream<Item=Event> + Unpin + Send + 'a {
        events.filter_map(move |event| {
            let items_map = self.items.clone();
            async move {
                match event {
                    Event::MessageCreate(message) => {
                        let index = items_map.get(&message.channel_id)
                            .and_then(|items_| items_.iter().enumerate().find(|(_, item)| (item.predicate)(&message)).map(|(i, _)| i));
                        if let Some(index) = index {
                            let channel = message.channel_id;
                            let items = items_map.get_mut(&channel);
                            if let Some(mut items) = items {
                                if let Some(item) = items.get_mut(index) {
                                    let MessageCreate(message) = *(message.clone());
                                    if item.sender.unbounded_send(message).is_ok() {
                                        if let Some(ref mut uses) = item.num_uses {
                                            *uses -= 1;
                                            if *uses == 0 {
                                                items.remove(index);
                                                if items.is_empty() {
                                                    items_map.remove(&channel);
                                                }
                                            }
                                            return None;
                                        }
                                    }
                                }
                            }
                        }
                        Some(Event::MessageCreate(message))
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
    pub fn wait_for<F: Fn(&Message) -> bool + Send + Sync + 'static>(&self, channel: ChannelId, predicate: F) -> WaitFor {
        WaitFor::new(self.wait_for_multiple(channel, Some(1), predicate))
    }

    /// Waits for multiple [`Message`]s that meet the predicate and is in the channel provided.
    ///
    /// This will wait *forever* until a [`Message`] is found `num_messages` times or the [`WaitForMultiple`] struct is dropped.
    ///
    /// You may want to timeout the future by using a function like `tokio::timer::timeout` or equivalent.
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
