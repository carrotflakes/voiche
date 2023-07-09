import "./polyfill.js";
import init, { Processor } from "./pkg/demo.js";

class Voiche extends AudioWorkletProcessor {
  constructor(options) {
    super();

    init(WebAssembly.compile(options.processorOptions.wasmData)).then(() => {
      this.processor = new Processor();
      this.port.postMessage({ type: "initialized" });
    });

    this.port.onmessage = async (ev) => {
      if (ev.data.type === "initialize") {
        await init(WebAssembly.compile(ev.data.data));
        this.processor = new Processor();
        this.port.postMessage({ type: "initialized" });

        // const self = this;
        // const imports = {
        //   wbg: {
        //     __wbindgen_throw(ptr, len) {
        //       throw new Error(
        //         "wbindgen_throw: " +
        //           String.fromCharCode.apply(
        //             "",
        //             new Uint8Array(
        //               self.instance.exports.memory.buffer
        //             ).subarray(ptr, ptr + len)
        //           )
        //       );
        //     },
        //   },
        // };
        // WebAssembly.instantiate(ev.data.bytes, imports).then((w) => {
        //   this.instance = w.instance;
        //   console.log(w);
        //   this.processor = new Processor(1.0);
        //   this.port.postMessage({ type: "initialized" });
        // });
      }

      if (ev.data.type === "setPitch" && typeof ev.data.pitch === "number") {
        this.processor.set_pitch(ev.data.pitch);
      }
      if (
        ev.data.type === "setFormant" &&
        typeof ev.data.formant === "number"
      ) {
        this.processor.set_formant(ev.data.formant);
      }
    };
  }

  process(inputs, outputs) {
    const input = inputs[0][0];
    const output = outputs[0][0];

    if (!input) return true;
    if (!this.processor) {
      return true;
    }

    this.processor.process(input);
    for (let i = 0; i < input.length; ++i) {
      output[i] = input[i];
    }

    return true;
  }
}

registerProcessor("voiche", Voiche);
