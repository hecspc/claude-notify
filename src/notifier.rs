pub trait Notifier {
    fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn name(&self) -> &str;
}
