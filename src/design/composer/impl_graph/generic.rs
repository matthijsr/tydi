




/*
#[derive(Clone, PartialEq)]
pub struct StreamletComp {
    key: NodeKey,
    streamlet_handle: Option<StreamletHandle>,
    streamlet: Streamlet,
}

impl StreamletComp {
    /// Construct a new instance.
    pub fn new(key: NodeKey, streamlet_handle: StreamletHandle, streamlet: Streamlet) -> Self {
        StreamletComp {
            key,
            streamlet_handle,
            streamlet: streamlet.clone()}
    }

    /// Return the key of this instance.
    pub fn key(&self) -> NodeKey {
        self.key.clone()
    }

    /// Return a reference to the streamlet this instance instantiates.
    pub fn streamlet(&self) -> Option<StreamletHandle> {
        self.streamlet_handle.clone()
    }

    pub fn get_interface(&self, project: &Project, key: IFKey) -> Result<Interface> {
        Ok(project
            .get_streamlet(self.streamlet())?
            .get_interface(key)?
            .clone())
    }
}

impl Debug for StreamletComp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} : {}", self.key, self.streamlet)
    }
}

impl GenericComponent for StreamletComp {

    fn key(&self) -> ComponentKey {
        self.key.clone()
    }

    /// Return an iterator over the interfaces of this Streamlet.
    fn interfaces(&self) -> Box<dyn Iterator<Item = &Interface>> {
        Box::new(self.interfaces.iter().map(|(_, i)| i))
    }

    fn get_interface(&self, key: IFKey) -> Result<Interface> {
        match self.interfaces.get(&key) {
            None => Err(Error::InterfaceError(format!(
                "Interface {} does not exist for Streamlet  {}.",
                key,
                self.identifier()
            ))),
            Some(iface) => Ok(iface.clone()),
        }
    }

    fn get_implementation(&self) -> Option<Rc<ImplementationGraph>> {
        self.implementation.clone()
    }
}*/