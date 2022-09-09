pub enum OffthreadAction<T> {
    Start(flume::Receiver<anyhow::Result<T>>),
    Error(String),
    Loaded(T),
}

impl<T: Send + 'static> std::ops::Deref for OffthreadAction<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Start(_) | Self::Error(_) => unreachable!(),
            Self::Loaded(inner) => inner,
        }
    }
}

impl<T: Send + 'static> std::ops::DerefMut for OffthreadAction<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Start(_) | Self::Error(_) => unreachable!(),
            Self::Loaded(inner) => inner,
        }
    }
}

impl<T: Send + 'static> OffthreadAction<T> {
    pub fn start(factory: impl FnOnce() -> anyhow::Result<T> + Send + 'static) -> Self {
        let (tx, rx) = flume::bounded(0);
        std::thread::spawn(move || {
            let res = factory();
        });
        Self::Start(rx)
    }

    pub fn try_recv(&mut self) -> bool {
        match self {
            Self::Start(rx) => {
                *self = match rx.try_recv() {
                    Ok(Ok(data)) => Self::Loaded(data),
                    Ok(Err(err)) => Self::Error(err.to_string()),
                    Err(_) => return false,
                }
            }
            Self::Error(_) | Self::Loaded(_) => {}
        }
        true
    }

    pub const fn is_loaded(&self) -> bool {
        !matches!(self, Self::Start { .. })
    }

    pub fn wait_for_loaded(&mut self) -> anyhow::Result<&mut T> {
        match self {
            Self::Start(rx) => {
                while !self.try_recv() {
                    std::thread::yield_now()
                }
                self.wait_for_loaded()
            }
            Self::Error(err) => Err(anyhow::anyhow!("{err}")),
            Self::Loaded(ok) => Ok(ok),
        }
    }
}
