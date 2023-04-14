import { Component, Input } from '@angular/core';
import { ProfileConfig } from '../profile-config';

@Component({
  selector: 'app-config-editor',
  templateUrl: './config-editor.component.html',
  styleUrls: ['./config-editor.component.scss']
})
export class ConfigEditorComponent {
  @Input() profileConfig!: ProfileConfig;
}
