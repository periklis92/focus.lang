class IoHandler {
    /**
     * @type EventTarget
     */
    eventTarget;
    constructor() {
        this.eventTarget = new EventTarget();
    }
}

const ioHandler = new IoHandler();

/**
 * 
 * @param {string} log 
 */
function log(log) {
    ioHandler.eventTarget.dispatchEvent(new CustomEvent("log", { detail: log }));
}