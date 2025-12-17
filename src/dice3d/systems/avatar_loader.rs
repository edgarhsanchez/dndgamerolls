//! Avatar loader system
//!
//! This module provides async loading of profile images from URLs (GitHub, Google, etc.)
//! Images are loaded in background threads and converted to Bevy textures.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::ui::widget::ImageNode;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

/// Size to resize avatars to (square)
const AVATAR_SIZE: u32 = 64;

/// Resource that manages avatar loading state
#[derive(Resource, Default)]
pub struct AvatarLoader {
    /// URLs that are currently being loaded
    loading: HashMap<String, ()>,
    /// Loaded image data ready to be converted to textures
    completed: Arc<Mutex<Vec<CompletedAvatar>>>,
    /// URLs that failed to load
    failed: Arc<Mutex<Vec<String>>>,
    /// Cache of loaded avatar textures
    pub cache: HashMap<String, Handle<Image>>,
    /// Set of URLs that have failed
    pub failed_urls: HashMap<String, ()>,
}

/// Completed avatar data from background thread
struct CompletedAvatar {
    url: String,
    image_data: Vec<u8>,
    width: u32,
    height: u32,
}

/// Component to mark an entity as waiting for an avatar
#[derive(Component)]
pub struct AvatarImage {
    pub url: String,
    pub loaded: bool,
    pub failed: bool,
}

impl AvatarLoader {
    /// Request an avatar to be loaded from a URL
    pub fn request(&mut self, url: &str) {
        // Skip if already loading, cached, or failed
        if self.loading.contains_key(url)
            || self.cache.contains_key(url)
            || self.failed_urls.contains_key(url)
        {
            return;
        }

        // Skip empty URLs
        if url.is_empty() {
            return;
        }

        // Mark as loading
        self.loading.insert(url.to_string(), ());

        // Clone for the thread
        let url_clone = url.to_string();
        let completed = Arc::clone(&self.completed);
        let failed = Arc::clone(&self.failed);

        // Spawn background thread to load the image
        thread::spawn(move || {
            if let Some(avatar) = load_avatar_from_url(&url_clone) {
                let mut completed_lock = completed.lock().unwrap();
                completed_lock.push(avatar);
            } else {
                // Mark as failed
                let mut failed_lock = failed.lock().unwrap();
                failed_lock.push(url_clone);
            }
        });
    }

    /// Check if an avatar is available in cache
    pub fn get(&self, url: &str) -> Option<Handle<Image>> {
        self.cache.get(url).cloned()
    }

    /// Check if an avatar is currently loading
    pub fn is_loading(&self, url: &str) -> bool {
        self.loading.contains_key(url)
    }

    /// Check if an avatar failed to load
    pub fn has_failed(&self, url: &str) -> bool {
        self.failed_urls.contains_key(url)
    }
}

/// Load an avatar from a URL (runs in background thread)
fn load_avatar_from_url(url: &str) -> Option<CompletedAvatar> {
    // Use reqwest blocking client
    let response = reqwest::blocking::Client::new()
        .get(url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let bytes = response.bytes().ok()?;

    // Decode the image
    let img = image::load_from_memory(&bytes).ok()?;

    // Resize to standard avatar size
    let resized = img.resize_exact(
        AVATAR_SIZE,
        AVATAR_SIZE,
        image::imageops::FilterType::Lanczos3,
    );

    // Convert to RGBA8
    let rgba = resized.to_rgba8();
    let (width, height) = rgba.dimensions();

    Some(CompletedAvatar {
        url: url.to_string(),
        image_data: rgba.into_raw(),
        width,
        height,
    })
}

/// System to process completed avatar loads and create textures
pub fn process_avatar_loads(
    mut avatar_loader: ResMut<AvatarLoader>,
    mut images: ResMut<Assets<Image>>,
) {
    // Get completed avatars
    let completed: Vec<CompletedAvatar> = {
        let mut lock = avatar_loader.completed.lock().unwrap();
        std::mem::take(&mut *lock)
    };

    // Get failed avatars
    let failed: Vec<String> = {
        let mut lock = avatar_loader.failed.lock().unwrap();
        std::mem::take(&mut *lock)
    };

    // Process each completed avatar
    for avatar in completed {
        // Remove from loading set
        avatar_loader.loading.remove(&avatar.url);

        // Create Bevy Image
        let image = Image::new(
            bevy::render::render_resource::Extent3d {
                width: avatar.width,
                height: avatar.height,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            avatar.image_data,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );

        // Add to asset system
        let handle = images.add(image);

        // Cache the handle
        avatar_loader.cache.insert(avatar.url, handle);
    }

    // Process failed avatars
    for url in failed {
        avatar_loader.loading.remove(&url);
        avatar_loader.failed_urls.insert(url, ());
    }
}

/// System to update UI images when avatars are loaded
pub fn update_avatar_images(
    avatar_loader: Res<AvatarLoader>,
    mut query: Query<(&mut ImageNode, &mut AvatarImage)>,
) {
    for (mut ui_image, mut avatar) in query.iter_mut() {
        if avatar.loaded || avatar.failed {
            continue;
        }

        if let Some(handle) = avatar_loader.get(&avatar.url) {
            ui_image.image = handle;
            avatar.loaded = true;
        } else if avatar_loader.has_failed(&avatar.url) {
            avatar.failed = true;
        }
    }
}

/// System to request avatar loading for AvatarImage components
pub fn request_avatars(
    mut avatar_loader: ResMut<AvatarLoader>,
    query: Query<&AvatarImage, Added<AvatarImage>>,
) {
    for avatar in query.iter() {
        if !avatar.url.is_empty() {
            avatar_loader.request(&avatar.url);
        }
    }
}
