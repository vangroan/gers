import "gers.input" for Keyboard, Mouse
import "gers.window" for Signal

class Game {
  // Called by engine to get the entry point.
  static handler_ { __handler }

  static deltaTime { __dt }
  static deltaTime_=(dt) { __dt = dt  }

  /* Signal emitted when the application encounters an error. */
  static onError { __error }

  // static run(handler) {
  //   __handler = handler
  // }

  /* Returns true if the game has registered an error handler. */
  static hasErrorHandler { !__error.isEmpty }

  /**
   * Hook for game engine to send errors to scripts, to be
   * handled by script code instead of aborting the application.
   */
  static sendError_(error) {
    __error.send(error)
  }

  /**
   * Creates a game instance.
   *
   * The game application is treated as a singleton. Only
   * one instance is allowed.
   *
   * # Errors
   *
   * Aborts the fiber if a game instance 
   *
   * # Example
   *
   * ```
   * class MyGame is Game {
   *   construct new() {
   *     super() // <-- Call this
   *   }
   * }
   * ```
   */
  construct new() {
    // We only allow setup once.
    if (__handler != null) {
      Fiber.abort("Game instance has already been setup")
    }

    // Store the instantiated game object as a singleton.
    __handler = this

    // Initialise delta time.
    __dt = 0.16

    // Signals
    __error = Signal.new()
  }

  // Override me
  init() {}

  // Override me
  update() {}

  // Override me
  draw() {}

  // Per frame update
  process_() {
    update()

    Keyboard.emitChars_()

    // Keyboard events should be emitted first
    // because of modifier keys. (Shift, Ctrl, Alt)
    Mouse.emitButtons_()
  }
}
