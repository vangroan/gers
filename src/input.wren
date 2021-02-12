
foreign class Input {
  construct new() {}

  foreign isPressed(keycode)
}

class KeyboardInput {
  // Subscribes the given function to receive
  // keyboard events as UTF-8 characters.
  static subscribeChar(fn) {
    ensure_()

    __subs.add(fn)
  }

  // Pushes the given character, as UTF-8 string, unto
  // the character queue.
  //
  // User keyboard input intended for GUI usage,
  // like text editing.
  //
  // *Called from foreign*
  static pushChar_(char) {
    __chars.add(char)
  }

  // Call subscribers with queued characters.
  static emitChars() {
    for (sub in __subs) {
      for (char in __chars) {
        sub.call(char)
      }
    }

    // Flush!
    __chars.clear()
  }

  // Indicates whether the given key is pressed or not.
  static isKeyPressed(keycode) {
    // Map returns null on non-existant key.
    var state = __state[keycode]

    return state == true
  }

  // *Called from foreign*
  static setKeyPress_(keycode) {
    // System.print("setKeyPress_(%(keycode))")
    __state[keycode] = true
  }

  // *Called from foreign*
  static setKeyRelease_(keycode) {
    __state.remove(keycode)
  }

  // Initialise the keyboard state.
  //
  // Maps scancode to pressed state. Pressed is
  // true and released is false.
  static ensure_() {
    if (__state == null) {
      __state = {}
    }

    if (__subs == null) {
      __subs = []
    }

    if (__chars == null) {
      __chars = []
    }
  }

  // *Called from foreign*
  static clear_() {
    __state.clear()
  }
}

KeyboardInput.ensure_()
