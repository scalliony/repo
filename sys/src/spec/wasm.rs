pub trait Store {
    type T;
    fn new(data: Self::T) -> Self;
    fn data(&mut self) -> &mut Self::T;
}

#[derive(Clone)]
pub struct LinkExport {
    pub name: &'static str,
    pub required: bool,
    pub value: ExportType,
}
#[derive(Clone)]
pub enum ExportType {
    UnitFunc
}
pub static MAY_EXPORT_START: LinkExport = LinkExport {
    name: "_start",
    required: false,
    value: ExportType::UnitFunc,
};
