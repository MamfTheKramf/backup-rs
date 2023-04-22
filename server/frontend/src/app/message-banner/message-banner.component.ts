import { Component, ElementRef, OnInit, ViewChild } from '@angular/core';
import { MessageService } from '../message.service';
import { Message, MessageType } from 'Message';

@Component({
  selector: 'app-message-banner',
  templateUrl: './message-banner.component.html',
  styleUrls: ['./message-banner.component.scss']
})
export class MessageBannerComponent implements OnInit {
  infoMsg = '';
  errorMsg = '';

  @ViewChild('InfoBanner')
  infoBanner!: ElementRef;

  @ViewChild('ErrorBanner')
  errorBanner!: ElementRef;

  constructor(private readonly messageService: MessageService) {}

  private hideBanner(banner?: HTMLElement): void {
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    if (!banner || !banner.classList) {
      return;
    }
    banner.classList.add('hidden');
  }

  handleMessage(msg: Message): void {
    console.log(msg);

    this.hideBanner(this.infoBanner.nativeElement as HTMLElement);
    this.hideBanner(this.errorBanner.nativeElement as HTMLElement);

    let banner: HTMLElement | undefined;

    if (msg.type == MessageType.Info) {
      this.infoMsg = msg.msg;
      banner = this.infoBanner.nativeElement as HTMLElement;
    }
    else {
      this.errorMsg = msg.msg;
      banner = this.errorBanner.nativeElement as HTMLElement;
    }

    banner.classList.remove('hidden');
    setTimeout(() => this.hideBanner(banner), 5000);
  }

  ngOnInit(): void {
    this.messageService.subscribe(this.handleMessage.bind(this));
  }
}
