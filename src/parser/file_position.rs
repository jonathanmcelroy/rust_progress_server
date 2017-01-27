#[derive(Serialize, Deserialize)]
pub struct FilePosition {
    row: u32,
    column: u32,
}

impl FilePosition { 
    pub fn new() -> Self {
        FilePosition { row: 0, column: 0 }
    }
}

pub struct FilePositionM<T> {
    file_position: FilePosition,
    inner_type: T,
}

impl<T> FilePositionM<T> {
    pub fn new(inner_type: T) -> Self {
        FilePositionM { file_position: FilePosition::new(), inner_type }
    }

    pub fn from(self) -> T {
        self.inner_type
    }
}

pub fn wrap<T>(inner: T) -> FilePositionM<T> {
    FilePositionM::new(inner)
}
