
foreign class Shader {
  /* Basic default shader. */
  static default { __default }

  /* Hook for engine to set the default shader. */
  static default_=(value) { __default = value }

  foreign static compile(device, vertex, fragment)
}
