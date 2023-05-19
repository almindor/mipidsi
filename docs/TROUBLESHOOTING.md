# Display troubleshooting

## Display stays black/blank

### Backlight pin

This driver does **NOT** handle the backlight pin to keep the code simpler. Users must control the backlight manually. First thing to try is to see if setting the backlight pin to high fixes the issue.

### Transport misconfiguration (e.g. SPI)

Make sure that the transport layer is configured correctly. Typical mistakes are the use of wrong SPI MODE or too fast transfer speeds that are not supported by the display
