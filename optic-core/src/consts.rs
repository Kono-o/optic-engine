// Asset paths
pub const ASSET: &str = "opt/";
pub const TEMP: &str = "opt/temp/";
pub const SHDR_ASSET: &str = "opt/shdr/";
pub const MESH_ASSET: &str = "opt/mesh/";
pub const TXTR_ASSET: &str = "opt/txtr/";

// File extensions
pub const VERT: &str = "vert";
pub const FRAG: &str = "frag";
pub const GLSL: &str = "glsl";
pub const OBJ: &str = "obj";
pub const PNG: &str = "png";

// Optic file format extensions
pub const OSHDR: &str = "oshdr";
pub const OMESH: &str = "omesh";
pub const OTXTR: &str = "otxtr";

// Magic signature for all optic engine binary cache files (8 bytes) — never changes
pub const OPTIC_MAGIC: [u8; 8] = *b"/0PTIC_x";
/// Version of the binary cache format — bump this when the layout after the header changes.
pub const OPTIC_CACHE_VERSION: u16 = 1;

// Shader type sub-discriminators
pub const SHADER_PIPELINE: u8 = 0;
pub const SHADER_COMPUTE: u8 = 1;

// Mesh attribute flags (bitmask)
pub const MESH_FLAG_HAS_NORMALS: u8 = 0b0001;
pub const MESH_FLAG_HAS_UVS: u8 = 0b0010;
