import { Component, Input } from '@angular/core';
import { ProfileConfig } from 'src/app/profile-config';

@Component({
  selector: 'app-config-info',
  templateUrl: './config-info.component.html',
  styleUrls: ['./config-info.component.scss']
})
export class ConfigInfoComponent {
  @Input() profileConfig!: ProfileConfig;

}
