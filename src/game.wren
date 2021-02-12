import "input" for KeyboardInput

class Game {
  static input { __input }
  static input_=(input) { __input = input }
  events { __events }

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

    __events = []
    __iter = null
  }

  // Push an event onto th event queue.
  //
  // Intended to be called from Rust.
  static push_event_(event) {
    __events.add(event)
  }

  static clear_events_() {
     __events.clear()
     __iter = null
  }

  poll() { __iter = __events.iterate(__iter) }

  event() { __events.iteratorValue(__iter) }

  // Override me
  init() {}

  // Override me
  update() {}

  // Per frame update
  process_() {
    update()

    KeyboardInput.emitChars()
  }
}
