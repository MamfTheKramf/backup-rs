import { ComponentFixture, TestBed } from '@angular/core/testing';

import { MessageBannerComponent } from './message-banner.component';

describe('MessageBannerComponent', () => {
  let component: MessageBannerComponent;
  let fixture: ComponentFixture<MessageBannerComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ MessageBannerComponent ]
    })
    .compileComponents();

    fixture = TestBed.createComponent(MessageBannerComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
