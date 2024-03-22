use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::mesh::{MeshVertexBufferLayout, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{
    AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};

/// A list of lines with a start and end position
#[derive(Debug, Clone)]
pub struct LineList {
    pub(crate) lines: Vec<(Vec3, Vec3)>,
}

impl From<LineList> for Mesh {
    fn from(line: LineList) -> Self {
        let vertices = line.lines.into_iter().flat_map(|(a, b)| [a, b]).collect::<Vec<_>>();

        Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::RENDER_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    }
}

#[derive(Asset, TypePath, Default, AsBindGroup, Debug, Clone)]
pub struct LineMaterial {
    #[uniform(0)]
    pub(crate) color: Color,
}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/line_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}
