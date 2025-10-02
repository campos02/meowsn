pub fn run_future<R, T>(
    handle: tokio::runtime::Handle,
    future: impl Future<Output = R> + Send + 'static,
    sender: std::sync::mpsc::Sender<T>,
    mut result_message: impl FnMut(R) -> T + Send + 'static,
) where
    R: Send + 'static,
    T: Send + 'static,
{
    handle.spawn(async move {
        let result = future.await;
        let _ = sender.send(result_message(result));
    });
}
