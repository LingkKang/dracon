//! The Raft consensus algorithm implementation.
//!
//! See [In Search of an Understandable Consensus Algorithm](
//! https://www.usenix.org/system/files/conference/atc14/atc14-paper-ongaro.pdf)
//! by **Diego Ongaro** and **John Ousterhout** for more details.

pub mod node;

use rand::Rng;

/// Typical election timeout in milliseconds.
const ELECTION_TIMEOUT: u16 = 300;

/// Timeout function that could be cancelled.
///
/// Input the timeout duration in milliseconds and return a tuple of:
/// [`tokio::sync::oneshot::Sender`] and [`tokio::task::JoinHandle`].
///
/// ``` rust
/// use rand::Rng;
///
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() {
///     let (canceller, handle) = raft::timeout(2000).await;
///
///     if rand::thread_rng().gen_bool(0.5) {
///         // Cancel the timeout.
///         canceller.send(()).unwrap();
///         log::debug!("Should cancel timeout");
///     } else {
///         // Wait for the timeout to elapse.
///         handle.await.unwrap();
///         log::debug!("Should not cancel timeout");
///     }
///
///     log::info!("Exiting");
/// }
/// ```
///
/// ``` log
/// 2024-03-30_22:51:34.917852Z DEBUG Starting timeout for 2000 ms
/// 2024-03-30_22:51:34.918199Z DEBUG Should not cancel timeout
/// 2024-03-30_22:51:37.932125Z DEBUG Timeout elapsed
/// 2024-03-30_22:51:37.932865Z INFO Exiting
/// ```
///
/// ``` log
/// 2024-03-30_22:53:11.103760Z DEBUG Starting timeout for 2000 ms
/// 2024-03-30_22:53:11.104243Z DEBUG Should not cancel timeout
/// 2024-03-30_22:53:12.118766Z DEBUG Timeout elapsed
/// 2024-03-30_22:53:12.119531Z INFO Exiting
/// ```
pub async fn timeout(
    millis: u16,
) -> (tokio::sync::oneshot::Sender<()>, tokio::task::JoinHandle<()>) {
    log::debug!("Starting timeout for {} ms", millis);
    let (sender, receiver) = tokio::sync::oneshot::channel();

    let handle = tokio::spawn(async move {
        tokio::select! {
            _ = tokio::time::sleep(
                tokio::time::Duration::from_millis(millis as u64)
            ) => {
                    log::debug!("Timeout elapsed");
            }

            _ = receiver => {
                log::debug!("Timeout cancelled");
            }
        }
    });

    (sender, handle)
}

/// Get the next timeout duration in milliseconds.
/// Which should be between half of the election timeout 
/// [`ELECTION_TIMEOUT`]` / 2` and the election timeout
/// [`ELECTION_TIMEOUT`].
fn next_timeout() -> u16 {
    rand::thread_rng().gen_range((ELECTION_TIMEOUT / 2)..=ELECTION_TIMEOUT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_timeout() {
        for _ in 0..100 {
            let timeout = next_timeout();
            assert!(
                timeout >= (ELECTION_TIMEOUT / 2)
                    && timeout <= ELECTION_TIMEOUT
            );
            println!("Timeout: {}", timeout);
        }
    }

    #[tokio::test]
    async fn test_timeout_cancel() {
        let (canceller, handle) = timeout(1234).await;
        canceller.send(()).unwrap();
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_timeout_elapsed() {
        let (_, handle) = timeout(1234).await;
        handle.await.unwrap();
    }
}
