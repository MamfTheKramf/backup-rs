import { Injectable } from '@angular/core';
import { Message } from 'Message';

export type SubscriberCallback = (msg: Message) => unknown;


@Injectable({
  providedIn: 'root'
})
export class MessageService {

  private subscribers: (SubscriberCallback)[] = [];

  subscribe(callback: SubscriberCallback): void {
    this.subscribers.push(callback);
  }

  sendMsg(msg: Message): void {
    this.subscribers.forEach(callback => callback(msg));
  }
}
