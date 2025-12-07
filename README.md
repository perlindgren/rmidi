# RMIDI - Rust Midi Abstractions

The aim is to provide a simple and safe abstractions for writing midi applications in Rust. The API takes inspiration from Apples 'core-midi' and corresponding low-level bindings, while abstracting away the implementation details.

The hope here is that once minimal functionality is implemented and works, backends for Windows (WinAPI/MM) and Linux will be added (if at all possible).

## Design approach

To provide a backend agnostic API, MIDI sources and destinations are identified by index provided by the backend. Indexes are used for connecting and disconnecting, and notifications from the backend once devices are added or removed.

Attaching sources (MIDI input to PC) and destinations (MIDI output from PC) is done manually. The user provides a callback in the former case.

If a connected device is detached, sending data to the destination is still allowed (to ensure robustness). Once the device is re-attached, sending data resumes without user intervention. Similarly a connected source, if detached does not provide any input, and is resumed when re-attached.

The exact form for notifications is still to be determined. The idea is that the user provides a callback function on creating the interface. The notifications will be a vector of events, indicting sources/destinations being added or removed. The indexing is managed by the underlying library, and ensured to be stable while connected. (However there is no requirement that sources/destinations follows sequential numbering, that is the case only at initialization).

Familiar (human readable) names are provided by the API, allowing configurations to be serialized, stored, de-serialized to attempt to recover MIDI session state. (There is of course no guarantee that the same set of MIDI sources/destinations are physically connected.)

## Example

The crate comes with a set of examples to showcase the functionality. The `egui_nux`, implements the skeleton for the MIGHTY line of NUX devices, tested on the MIGHTY SPACE modelling amplifier.

## License

MIT/APACHE to your liking
