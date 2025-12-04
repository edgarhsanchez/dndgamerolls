# D&D Dice 3D Models

This folder is for custom 3D dice models. The application currently uses procedurally generated meshes, but you can replace them with custom FBX/GLTF models.

## Supported Formats
- **GLTF/GLB** (recommended for Bevy)
- **FBX** (requires conversion to GLTF)

## Expected Model Files

Place your dice models here with these names:
- `d4.glb` - Tetrahedron (4 faces)
- `d6.glb` - Cube (6 faces)
- `d8.glb` - Octahedron (8 faces)
- `d10.glb` - Pentagonal trapezohedron (10 faces)
- `d12.glb` - Dodecahedron (12 faces)
- `d20.glb` - Icosahedron (20 faces)
- `d100.glb` - Percentile die (same as D10 with 00-90)
- `dice_box.glb` - The rolling box/tray

## Model Requirements

### Scale
- Models should be approximately 1 unit in size
- Center the model at origin (0, 0, 0)

### Orientation
- Y-axis should be up
- Number "1" should face a consistent direction

### Materials
- Include PBR materials if possible
- Base color, metallic, roughness textures

## Converting FBX to GLTF

You can use Blender to convert FBX files:

1. Open Blender
2. File > Import > FBX
3. Select your FBX file
4. File > Export > glTF 2.0
5. Choose "GLB" format for a single file
6. Enable "Apply Modifiers"
7. Export

Or use the command-line tool `FBX2glTF`:
```bash
FBX2glTF -i input.fbx -o output.glb
```

## Creating Dice Models in Blender

### D4 (Tetrahedron)
1. Add > Mesh > Cone (vertices: 3, depth: 1)
2. Add numbers 1-4 on each face

### D6 (Cube)
1. Add > Mesh > Cube
2. Standard pip or number arrangement

### D8 (Octahedron)
1. Add > Mesh > Ico Sphere (subdivisions: 1)
2. Or create manually with 8 triangular faces

### D10 (Pentagonal Trapezohedron)
1. Create custom mesh with 10 kite-shaped faces
2. Numbers 0-9

### D12 (Dodecahedron)
1. Add > Mesh > Ico Sphere, then edit
2. Or use Add-on: "Add Mesh: Extra Objects"

### D20 (Icosahedron)
1. Add > Mesh > Ico Sphere (subdivisions: 1)
2. 20 triangular faces numbered 1-20

## Number Placement

Standard D&D dice have specific number arrangements:
- Opposite faces should sum to N+1 (for a dN die)
- D6: 1-6, 2-5, 3-4 are opposite
- D20: 1-20, 2-19, 3-18, etc. are opposite

## UV Mapping

For texture-based numbers:
1. Unwrap each face
2. Map to a texture atlas with all numbers
3. Or use individual textures per face

## Physics Colliders

The app generates convex hull colliders automatically, but for best results:
- Keep models convex or nearly convex
- Avoid thin protrusions
- Ensure watertight geometry
