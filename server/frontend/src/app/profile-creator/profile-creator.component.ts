import { Component, EventEmitter, Output } from '@angular/core';
import { ProfileConfig } from '../profile-config';
import { ApiServiceService } from '../api-service.service';

@Component({
  selector: 'app-profile-creator',
  templateUrl: './profile-creator.component.html',
  styleUrls: ['./profile-creator.component.scss']
})
export class ProfileCreatorComponent {
  @Output() created = new EventEmitter<ProfileConfig>();

  name = '';

  constructor(private readonly api: ApiServiceService) {}

  create(): void {
    console.log(this.name);
    if (!this.name) {
      return;
    }

    this.api.createBlankProfileConfig(this.name)
      .subscribe(config => {
        if (!config) {
          return;
        }

        this.created.emit(config);
      });
  }
}
