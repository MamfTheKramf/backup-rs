import { ComponentFixture, TestBed } from '@angular/core/testing';

import { SpecifierEditorComponent } from './specifier-editor.component';

describe('SpecifierEditorComponent', () => {
  let component: SpecifierEditorComponent;
  let fixture: ComponentFixture<SpecifierEditorComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ SpecifierEditorComponent ]
    })
    .compileComponents();

    fixture = TestBed.createComponent(SpecifierEditorComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
