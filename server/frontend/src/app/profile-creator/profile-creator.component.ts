import { Component, EventEmitter, Output } from '@angular/core';
import { ProfileConfig } from '../profile-config';

@Component({
  selector: 'app-profile-creator',
  templateUrl: './profile-creator.component.html',
  styleUrls: ['./profile-creator.component.scss']
})
export class ProfileCreatorComponent {
  @Output() created = new EventEmitter<ProfileConfig>();

  name = '';

  create(): void {
    console.log(this.name);
  }
}
