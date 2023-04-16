import { Component, Input } from '@angular/core';
import { ProfileConfig } from 'src/app/profile-config';

@Component({
  selector: 'app-interval-editor',
  templateUrl: './interval-editor.component.html',
  styleUrls: ['./interval-editor.component.scss']
})
export class IntervalEditorComponent {
  @Input() profileConfig!: ProfileConfig;
}
