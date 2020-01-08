use chashmap::CHashMap;
use dawn_model::{channel::Message, id::ChannelId};
use futures_channel::mpsc::UnboundedReceiver;
use futures_util::stream::Stream;
use pin_project::{pinned_drop, pin_project};

use std::{future::Future, pin::Pin, sync::Arc, task::{Context, Poll}};
use std::time::Instant;

use crate::listener::ListenerItem;

/// A struct that implements a [`Stream`] of [`Message`]s.
#[pin_project(PinnedDrop)]
pub struct WaitForMultiple {
    /// The channel from which this struct is waiting for a [`Message`] from.
    pub channel_id: ChannelId,
    pub(crate) created: Instant,
    pub(crate) items_map: Arc<CHashMap<ChannelId, Vec<ListenerItem>>>,
    pub(crate) num: Option<u8>,
    #[pin]
    pub(crate) receiver: UnboundedReceiver<Message>,
}

impl Stream for WaitForMultiple {
    type Item = Message;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<Self::Item>> {
        let wait_for = self.project();
        match wait_for.num {
            Some(num) if *num == 0 => Poll::Ready(None),
            _ => {
                let poll = wait_for.receiver.poll_next(ctx);
                if poll.is_ready() {
                    *wait_for.num = wait_for.num.map(|n| n - 1);
                }
                poll
            }
        }
    }
}

#[pinned_drop]
impl PinnedDrop for WaitForMultiple {
    fn drop(mut self: Pin<&mut Self>) {
        let channel_id = self.channel_id;
        let created = self.created;
        let items_map = &self.items_map;
        let num = self.num;
        match num {
            Some(0) => {}
            _ => if let Some(ref mut items) = items_map.get_mut(&channel_id) {
                items.retain(|item| item.created != created);
            }
        }
    }
}

/// A struct that implements [`Future`] which eventually returns a [`Message`].
#[pin_project]
pub struct WaitFor(#[pin] WaitForMultiple, bool);
impl WaitFor {
    pub(crate) fn new(wait_for: WaitForMultiple) -> Self {
        Self(wait_for, true)
    }
}
impl Future for WaitFor {
    type Output = Message;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        let wait_for = self.project();
        if *wait_for.1 {
            let poll = wait_for.0.poll_next(ctx);
            if poll.is_ready() {
                *wait_for.1 = false;
            }
            return poll.map(|o| o.unwrap())
        }
        Poll::Pending
    }
}
