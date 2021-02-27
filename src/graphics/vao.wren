
foreign class VertexArrayObject {
  /**
   * Required but unused constructor.
   *
   * `rust-wren` currently does not support
   * constructors that can return errors.
   */
  construct new_() {}
  foreign static new(device, vertices, indices)
}
