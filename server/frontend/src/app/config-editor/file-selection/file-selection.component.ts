import { Component, Input } from '@angular/core';
import { ProfileConfig } from 'src/app/profile-config';

@Component({
  selector: 'app-file-selection',
  templateUrl: './file-selection.component.html',
  styleUrls: ['./file-selection.component.scss']
})
export class FileSelectionComponent {
  @Input() profileConfig!: ProfileConfig;
}
