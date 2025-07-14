pub trait MappingExt<Res> {
    fn object_map(self) -> Res;
}

#[allow(dead_code)]
pub trait MappingCloneExt<T> {
    fn object_map_clone(&self) -> T;
}

impl<Source, Res> MappingCloneExt<Res> for Source
where
    Source: Clone + MappingExt<Res>,
{
    fn object_map_clone(&self) -> Res {
        self.clone().object_map()
    }
}

pub trait MappingExtraField1Ext<Res, Extra> {
    fn object_map_field(self, extra: Extra) -> Res;
}

#[allow(dead_code)]
pub trait MappingCloneExtraField1Ext<Res, Extra> {
    fn object_map_field_clone(&self, extra: &Extra) -> Res;
}

impl<Source, Res, Extra> MappingCloneExtraField1Ext<Res, Extra> for Source
where
    Source: Clone + MappingExtraField1Ext<Res, Extra>,
    Extra: Clone,
{
    fn object_map_field_clone(&self, extra: &Extra) -> Res {
        self.clone().object_map_field(extra.clone())
    }
}
