Enable logging using DEFMT_LOG=info

The brltty service claims the usb device, pop-desktop depends on it, so it cannot be removed, but must be disabled:
```bash
systemctl stop brltty-udev.service
sudo systemctl mask brltty-udev.service
systemctl stop brltty.service
systemctl disable brltty.service
```
