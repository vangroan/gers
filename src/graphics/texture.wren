
foreign class Texture {
  /** Allocates a new empty texture. */
  foreign static new(device, width, height)
  foreign static fromFile(device, filepath)
  foreign static fromColor(device, red, green, blue, alpha)
}
