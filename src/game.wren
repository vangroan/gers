import "input" for Keyboard

class Game {
  // Called by engine to get the entry point.
  static handler_ { __handler }

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
