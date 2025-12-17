# DnDGameRolls Bevy 0.17 + Material UI Migration Plan

## Overview

This document outlines the migration of DnDGameRolls from legacy Bevy UI APIs to Bevy 0.17.3 and bevy_material_ui.

## Migration Phases

### Phase 1: Bevy 0.17.3 API Compatibility

Fix all deprecated/removed APIs to work with Bevy 0.17.3.

### Phase 2: bevy_material_ui Integration

Replace custom UI components with Material Design 3 components.

---

## Phase 1: Bevy 0.17 API Changes

### 1. UI Bundle Replacements

| Old (Bevy 0.14/0.15) | New (Bevy 0.17) | Notes |
|---------------------|-----------------|-------|
| `TextBundle::from_section(text, style)` | `(Text::new(text), TextFont { font_size, .. }, TextColor(color))` | Text is now a component tuple |
| `NodeBundle { style, background_color, .. }` | `Node { width, height, .. }` + `BackgroundColor` | Style fields moved into Node |
| `ButtonBundle { style, background_color, .. }` | `(Button, Node { .. }, BackgroundColor)` | Button is a marker, style in Node |
| `ImageBundle { image: UiImage::new(h), style, .. }` | `(ImageNode::new(h), Node { .. })` | UiImage renamed to ImageNode |

### 2. Style → Node Migration

The `Style` component no longer exists. All layout properties moved to `Node`:

```rust
// OLD
NodeBundle {
    style: Style {
        width: Val::Px(100.0),
        height: Val::Px(50.0),
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(10.0)),
        margin: UiRect::vertical(Val::Px(5.0)),
        position_type: PositionType::Absolute,
        left: Val::Px(10.0),
        top: Val::Px(20.0),
        ..default()
    },
    background_color: BackgroundColor(Color::WHITE),
    ..default()
}

// NEW
(
    Node {
        width: Val::Px(100.0),
        height: Val::Px(50.0),
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(10.0)),
        margin: UiRect::vertical(Val::Px(5.0)),
        position_type: PositionType::Absolute,
        left: Val::Px(10.0),
        top: Val::Px(20.0),
        ..default()
    },
    BackgroundColor(Color::WHITE),
)
```

### 3. Text Changes

```rust
// OLD
TextBundle::from_section(
    "Hello",
    TextStyle {
        font_size: 16.0,
        color: Color::WHITE,
        ..default()
    },
)

// NEW
(
    Text::new("Hello"),
    TextFont {
        font_size: 16.0,
        ..default()
    },
    TextColor(Color::WHITE),
)
```

### 4. Image Changes

```rust
// OLD
ImageBundle {
    image: UiImage::new(handle),
    style: Style {
        width: Val::Px(24.0),
        height: Val::Px(24.0),
        ..default()
    },
    ..default()
}

// NEW
(
    ImageNode::new(handle),
    Node {
        width: Val::Px(24.0),
        height: Val::Px(24.0),
        ..default()
    },
)
```

### 5. Query Method Changes

| Old | New |
|-----|-----|
| `query.get_single()` | `query.single()` |
| `query.get_single_mut()` | `query.single_mut()` |
| `commands.entity(e).despawn_recursive()` | `commands.entity(e).despawn()` |

### 6. Import Changes

| Old Import | New Import |
|------------|------------|
| `bevy::render::texture::Image` | `bevy::prelude::Image` |
| `UiImage` | `ImageNode` |
| `Style` | (removed - use `Node` fields) |
| `TextStyle` | `TextFont` + `TextColor` |

### 7. Node Rect Calculations

```rust
// OLD
let rect = node.logical_rect(global_transform);

// NEW
let rect = node.physical_rect(global_transform, 1.0, UiRoundingMode::Nearest);
// Or use computed_node for layout info
```

---

## Phase 2: bevy_material_ui Component Mapping

### Component Replacements

| DnDGameRolls Custom | bevy_material_ui | Location |
|--------------------|------------------|----------|
| Tab buttons | `MaterialTabs` / `TabBuilder` | character_screen.rs |
| Action buttons | `MaterialButton` | character_screen.rs, settings.rs |
| Icon buttons | `MaterialIconButton` | character_screen.rs |
| Text input fields | `MaterialTextField` | character_screen.rs |
| Proficiency checkboxes | `MaterialCheckbox` | character_screen.rs |
| Zoom slider | `MaterialSlider` | setup.rs, camera.rs |
| Strength slider | `MaterialSlider` | throw_control/ui.rs |
| Color sliders | `MaterialSlider` | settings.rs |
| Settings modal | `MaterialDialog` | settings.rs |
| Character list | `MaterialList` | character_screen.rs |
| Stat groups | `MaterialCard` | character_screen.rs |
| Panel dividers | `MaterialDivider` | character_screen.rs |
| Scrollable areas | `ScrollContainer` | character_screen.rs |

### New Components Available

These bevy_material_ui components could enhance the UI:

- `MaterialSwitch` - For toggle options (instead of checkboxes for on/off)
- `MaterialChip` - For skill/proficiency tags
- `MaterialSnackbar` - For save confirmations, error messages
- `MaterialTooltip` - For help text on hover
- `MaterialProgress` - For loading indicators
- `TopAppBar` - For app header
- `MaterialBadge` - For unsaved indicator
- `MaterialMenu` - For context menus
- `MaterialSelect` - For dropdowns (class, race selection)
- `MaterialFab` - For primary actions (roll dice)

---

## Files to Migrate

### High Priority (Core UI)

1. **`src/dice3d/systems/character_screen.rs`** (5192 lines)
   - Tab bar, character list, stat panels
   - Buttons, text fields, checkboxes
   - Most complex file

    _Update:_ this has been split into a module folder under `src/dice3d/systems/character_screen/` (see `mod.rs`).

2. **`src/dice3d/systems/settings.rs`** (760 lines)
   - Settings modal dialog
   - Color picker sliders
   - OK/Cancel buttons

3. **`src/dice3d/systems/setup.rs`**
   - Initial UI setup
   - Zoom slider
   - Command input

4. **`src/dice3d/throw_control/ui.rs`** (96 lines)
   - Strength slider

### Medium Priority

5. **`src/dice3d/systems/camera.rs`**
   - Zoom slider interaction

6. **`src/dice3d/systems/contributors_screen.rs`**
   - Contributor list display

7. **`src/dice3d/types/ui.rs`** (466 lines)
   - UI component markers
   - May need updates for new component types

8. **`src/dice3d/types/icons.rs`**
   - Icon loading/display

### Low Priority

9. **`src/dice3d/systems/avatar_loader.rs`**
   - Avatar image loading

10. **`src/main.rs`**
    - System registration

---

## Migration Steps

### Step 1: Fix Imports
- Update all `use bevy::*` statements
- Add `use bevy::ui::widget::*` for ImageNode
- Remove Style imports, ensure Node is imported

### Step 2: Convert NodeBundle to Node tuples
- Replace `NodeBundle { style: Style { .. }, .. }` with `(Node { .. }, BackgroundColor(..))`

### Step 3: Convert TextBundle to Text tuples
- Replace `TextBundle::from_section(..)` with `(Text::new(..), TextFont { .. }, TextColor(..))`

### Step 4: Convert ButtonBundle to Button tuples
- Replace `ButtonBundle { .. }` with `(Button, Node { .. }, BackgroundColor(..))`

### Step 5: Convert ImageBundle to ImageNode tuples
- Replace `ImageBundle { image: UiImage::new(..), .. }` with `(ImageNode::new(..), Node { .. })`

### Step 6: Fix query methods
- Replace `get_single()` with `single()`
- Replace `despawn_recursive()` with `despawn()`

### Step 7: Add bevy_material_ui dependency
- Add to Cargo.toml
- Import prelude

### Step 8: Replace custom components with Material UI
- Incrementally replace each component type
- Test after each replacement

---

## Estimated Effort

| Task | Files | Estimated Changes |
|------|-------|-------------------|
| Bundle conversions | 8 | ~500 replacements |
| Text conversions | 8 | ~200 replacements |
| Query method fixes | 5 | ~50 replacements |
| Import fixes | 10 | ~30 changes |
| Material UI integration | 6 | ~300 component swaps |

**Total: ~1000+ code changes across 10 files**

---

## Testing Checklist

After migration, verify:

- [ ] App starts without errors
- [ ] Tab navigation works
- [ ] Character list displays and is selectable
- [ ] Stat editing works (text fields)
- [ ] Checkboxes toggle
- [ ] Sliders drag correctly
- [ ] Settings modal opens/closes
- [ ] Color picker works
- [ ] Save/load characters works
- [ ] Dice rolling UI works
- [ ] Theme colors look correct
