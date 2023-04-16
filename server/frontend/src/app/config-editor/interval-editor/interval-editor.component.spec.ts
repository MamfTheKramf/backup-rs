import { ComponentFixture, TestBed } from '@angular/core/testing';

import { IntervalEditorComponent } from './interval-editor.component';

describe('IntervalEditorComponent', () => {
  let component: IntervalEditorComponent;
  let fixture: ComponentFixture<IntervalEditorComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ IntervalEditorComponent ]
    })
    .compileComponents();

    fixture = TestBed.createComponent(IntervalEditorComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
