pub trait Store {
    type T;
    fn new(state: Self::T) -> Self;
    fn state(&self) -> &Self::T;
    fn state_mut(&mut self) -> &mut Self::T;
}
pub trait StoreRef<'a, S>
where
    S: Store + 'a,
{
    fn store(&self) -> &S;
    fn store_mut(&mut self) -> &mut S;
    #[inline]
    fn state(&'a self) -> &'a S::T {
        self.store().state()
    }
    #[inline]
    fn state_mut(&'a mut self) -> &'a mut S::T {
        self.store_mut().state_mut()
    }
}

#[derive(Clone)]
pub struct LinkExport {
    pub name: &'static str,
    pub required: bool,
    pub value: ExportType,
}
#[derive(Clone)]
pub enum ExportType {
    UnitFunc,
}
pub static MAY_EXPORT_START: LinkExport =
    LinkExport { name: "_start", required: false, value: ExportType::UnitFunc };
