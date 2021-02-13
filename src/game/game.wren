import "input" for Keyboard

class Game {
  // Called by engine to get the entry point.
  static handler_ { __handler }

  static deltaTime { __dt }
  static deltaTime_=(dt) { __dt = dt  }

  static run(handler) {
    __handler = handler
  }

  setup(handler) {
    // We only allow setup once.
    if (__handler != null) {
      Fiber.abort("Game instance has already been setup")
    }

    // Store the instantiated game object as a singleton.
    __handler = this

    // Initialise delta time.
    __dt = 0.16
  }

  // Override me
  init() {}

  // Override me
  update() {}

  // Per frame update
  process_() {
    update()

    Keyboard.emitChars()
  }
}
