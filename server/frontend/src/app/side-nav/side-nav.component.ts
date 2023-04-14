import { Component, Input } from '@angular/core';
import { ProfileConfig } from '../profile-config';

@Component({
  selector: 'app-side-nav',
  templateUrl: './side-nav.component.html',
  styleUrls: ['./side-nav.component.scss']
})
export class SideNavComponent {
  @Input() profileConfigs: ProfileConfig[] = [];
}
