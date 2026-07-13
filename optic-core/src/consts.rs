//! Constants for asset paths, cache magic, shader/mesh metadata.
//!
//! These are used by `optic_file` and `optic_render` to resolve
//! asset locations and validate cache files.

/// Root asset directory.
pub const ASSET: &str = "opt/";
/// Temporary asset directory.
pub const TEMP: &str = "opt/temp/";
/// Shader asset directory.
pub const SHDR_ASSET: &str = "opt/shdr/";
/// Mesh asset directory.
pub const MESH_ASSET: &str = "opt/mesh/";
/// Texture asset directory.
pub const TXTR_ASSET: &str = "opt/txtr/";

/// Vertex shader file extension.
pub const VERT: &str = "vert";
/// Fragment shader file extension.
pub const FRAG: &str = "frag";
/// GLSL shader file extension.
pub const GLSL: &str = "glsl";
/// Wavefront OBJ mesh file extension.
pub const OBJ: &str = "obj";
/// PNG image file extension.
pub const PNG: &str = "png";

/// Optic cached shader extension.
pub const OSHDR: &str = "oshdr";
/// Optic cached mesh extension.
pub const OMESH: &str = "omesh";
/// Optic cached texture extension.
pub const OTXTR: &str = "otxtr";
/// Optic cached font extension.
pub const OFONT: &str = "ofont";
/// Optic cached sound extension.
pub const OMUSIC: &str = "omusic";

/// Magic signature for all optic engine binary cache files (8 bytes).
///
/// This never changes. Every binary cache file starts with these bytes.
pub const OPTIC_MAGIC: [u8; 8] = *b"/0PTIC_x";

/// Version of the binary cache format.
///
/// Bump this when the layout after the header changes.
pub const OPTIC_CACHE_VERSION: u16 = 1;

/// Shader file sub-type discriminator for pipeline shaders.
pub const SHADER_PIPELINE: u8 = 0;
/// Shader file sub-type discriminator for compute shaders.
pub const SHADER_COMPUTE: u8 = 1;

/// Bitflag: mesh has normal data.
pub const MESH_FLAG_HAS_NORMALS: u8 = 0b0001;
/// Bitflag: mesh has UV data.
pub const MESH_FLAG_HAS_UVS: u8 = 0b0010;
