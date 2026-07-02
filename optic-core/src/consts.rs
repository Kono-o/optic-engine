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

// Magic signature for all optic engine binary cache files (16 bytes)
pub const OPTIC_MAGIC: [u8; 17] = *b"o/0ptiC+EngiNEx*_";
pub const CACHE_VERSION: u8 = 1;

// Asset type discriminators
pub const ASSET_TYPE_TEXTURE: u8 = 0;
pub const ASSET_TYPE_SHADER: u8 = 1;
pub const ASSET_TYPE_MESH: u8 = 2;

// Shader type sub-discriminators
pub const SHADER_PIPELINE: u8 = 0;
pub const SHADER_COMPUTE: u8 = 1;

// Mesh attribute flags (bitmask)
pub const MESH_FLAG_HAS_NORMALS: u8 = 0b0001;
pub const MESH_FLAG_HAS_UVS: u8 = 0b0010;
