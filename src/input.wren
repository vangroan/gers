

class Mouse {
  static logicalX { __logicalX }
  static logicalY { __logicalY }
  static physicalX { __physicalX }
  static physicalY { __physicalY }

  // Initialise the mouse state.
  static init_() {
    __state = {}
    __buttons = []
    __subs = []
    __logicalX = 0.0
    __logicalY = 0.0
    __physicalX = 0.0
    __physicalY = 0.0

    // Map of numbers to mouse button names.
    //
    // Determined by what gets sent from Rust
    // on mouse button state change.
    __btn_map = {
      1: "Left",
      2: "Middle",
      3: "Right",
    }
  }

  // Drains the character queue.
  static setPos_(lx, ly, px, py) {
    __logicalX = lx
    __logicalY = ly
    __physicalX = px
    __physicalY = py
  }

  // Indicates whether the given button is pressed or not.
  //
  // Note: When the state is undefined it is null.
  static isButtonPressed(button) { __state[button] == true }

  // Subscribes the given function to receive
  // mouse button state events.
  static onButton(fn) {
    __subs.add(fn)
  }

  // Call subscribers with queued button states.
  //
  // Drains the character queue.
  static emitButtons_() {
    for (sub in __subs) {
      for (button in __buttons) {
        sub.call(button[0], button[1])
      }
    }

    // Flush
    __buttons.clear()
  }

  // Pushes the given button state onto the
  // queue, to be handled by event subscribers.
  //
  // - `button` string identifying mouse button.
  // - `isPressed` true when pressed, false when released.
  //
  // *Called from foreign*
  static pushButton_(buttonId, isPressed) {
    var button = __btn_map[buttonId]

    var state = "Released"
    if (isPressed) {
      state = "Pressed"
    }

    // Queue for event listeners
    __buttons.add([button, state])

    // Map for polling state during frame update.
    __state[button] = isPressed
  }
}

Mouse.init_()


class Keyboard {
  // Subscribes the given function to receive
  // keyboard events as UTF-8 characters.
  //
  // # Example
  //
  // ```
  // Keyboard.subscribeChar {|char|
  //   System.print(char)
  // }
  // ```
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
  //
  // Drains the character queue.
  static emitChars_() {
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

  // Clears the keyboard state for the next frame.
  //
  // *Called from foreign*
  static clear_() {
    __state.clear()
  }
}

Keyboard.ensure_()
