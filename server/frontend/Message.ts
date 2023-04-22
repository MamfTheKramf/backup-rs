// Contains types for Messages

export enum MessageType {
    Info,
    Error,
}

export class Message {
    constructor(readonly type: MessageType, readonly msg: string) {}
}