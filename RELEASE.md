# Release

## Flatpak
To test the build for `Flatpak` and release to `Flathub`, head to [FLATPAK.md](../FLATPAK.md).

## Arch Linux
Update the [PKGBUILD](PKGBUILD) to include the new release number in `pkgrel` and `pkgver`.

## Windows
To compile for Windows, we need to configure Docker:

### Setup
To install Docker in Ubuntu:
```bash
sudo apt install docker-io
sudo systemctl enable docker
sudo systemctl start docker
```

We'll make use of an image in DockerHub.

```bash
docker pull mglolenstine/gtk4-cross:rust-gtk-4.6
```

Once it downloads, we need to create a container inside the project:
```bash
docker run -ti -v $(pwd):/mnt mglolenstine/gtk4-cross:rust-gtk-4.6
```

Once inside, we need to run `build` to build the project and `package` to package it into a zip file.

After that we'll have a `package.zip` in the root directory.

## macOS
macOS support is still on the way.