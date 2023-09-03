# Bevy cosmic edit

This is a fork of https://github.com/StaffEngineer/bevy_cosmic_edit/ because I expect to need to make minor changes. Some of these changes might get merged back into the original repo, but I don't want to wait for that to happen.

## Changes

- Remove `sys-locale` dep since cosmic-text already implements locale detection
- bevy_color_to_cosmic made more efficient
- (Regression) Remove `CosmicFontConfig` since I load fonts from a custom directory and then rely on cosmic-text's default configuration
- (Regression) Remove `set_text` method from bundles, since setting text will be set manually for my needs
