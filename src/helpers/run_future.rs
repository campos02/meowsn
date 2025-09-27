pub fn run_future<R, T>(
    future: impl Future<Output = R> + Send + 'static,
    sender: std::sync::mpsc::Sender<T>,
    result_message: impl Fn(R) -> T + Send + 'static,
) where
    R: Send + 'static,
    T: Send + 'static,
{
    tokio::spawn(async move {
        let result = future.await;
        let _ = sender.send(result_message(result));
    });
}
