import { Component, EventEmitter, Input, Output } from '@angular/core';
import { ProfileConfig } from '../profile-config';

@Component({
  selector: 'app-dashboard',
  templateUrl: './dashboard.component.html',
  styleUrls: ['./dashboard.component.scss']
})
export class DashboardComponent {
  @Input() profileConfigs: ProfileConfig[] = [];
  @Output() selected = new EventEmitter<ProfileConfig>();
  
  onSelect(config: ProfileConfig): void {
    this.selected.emit(config);
  }
}
