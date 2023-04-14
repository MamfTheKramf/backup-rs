import { Component, EventEmitter, Input, Output } from '@angular/core';
import { ProfileConfig } from '../profile-config';

@Component({
  selector: 'app-side-nav',
  templateUrl: './side-nav.component.html',
  styleUrls: ['./side-nav.component.scss']
})
export class SideNavComponent {
  @Input() profileConfigs: ProfileConfig[] = [];
  selectedUuid = '';
  @Output() selected = new EventEmitter<ProfileConfig>();

  select(config: ProfileConfig): void {
    this.selectedUuid = config.uuid;
    this.selected.emit(config);
  }

}
