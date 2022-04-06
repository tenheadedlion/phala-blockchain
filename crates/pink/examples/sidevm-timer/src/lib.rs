use pink_sidevm_env::{self as env, ocall_funcs_guest as ocall};

/// Resource ID. Think of it as a FD.
pub struct ResourceId(pub i32);

impl From<i32> for ResourceId {
    fn from(i: i32) -> Self {
        ResourceId(i)
    }
}

impl Drop for ResourceId {
    fn drop(&mut self) {
        let _ = ocall::close(self.0);
    }
}

mod sleep {
    use super::*;

    use core::pin::Pin;
    use std::future::Future;
    use std::task::{Context, Poll};
    use std::time::Duration;
    /// Resource ID. Think of it as a FD.
    pub struct ResourceId(pub i32);

    impl Drop for ResourceId {
        fn drop(&mut self) {
            let _ = ocall::close(self.0);
        }
    }

    pub struct Sleep {
        id: ResourceId,
    }

    pub fn sleep(duration: Duration) -> Sleep {
        let id = ocall::create_timer(duration.as_millis() as i32).expect("failed to create timer");
        Sleep { id: ResourceId(id) }
    }

    impl Future for Sleep {
        type Output = ();

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let rv = ocall::poll_read(self.id.0, &mut []).expect("Poll timer failed");
            match rv {
                env::Poll::Ready(_) => Poll::Ready(()),
                env::Poll::Pending => Poll::Pending,
            }
        }
    }
}

mod channel {
    use super::{ocall, ResourceId};
    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    };

    pub struct ChannelRx {
        res_id: ResourceId,
    }

    pub struct ChannelRxNext<'a> {
        ch: &'a ChannelRx,
    }

    impl ChannelRx {
        pub const fn new(res_id: ResourceId) -> Self {
            Self { res_id }
        }

        pub fn next(&self) -> ChannelRxNext {
            ChannelRxNext { ch: self }
        }
    }

    impl Future for ChannelRxNext<'_> {
        type Output = Option<Vec<u8>>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            ocall::poll(self.ch.res_id.0)
                .expect("Poll timer failed")
                .into()
        }
    }
}

fn message_rx() -> &'static channel::ChannelRx {
    static MSG_RX: channel::ChannelRx = channel::ChannelRx::new(ResourceId(0));
    &MSG_RX
}

#[pink_sidevm_env::main]
async fn main() {
    use pink_sidevm_env::ocall_funcs_guest as ocall;
    use std::time::Duration;

    ocall::enable_ocall_trace(true).unwrap();
    let msg_rx = message_rx();
    let _ = msg_rx.next().await;
    let message = msg_rx.next().await;
    assert_eq!(message, Some(b"foo".to_vec()));
    sleep::sleep(Duration::from_secs(3)).await
}
