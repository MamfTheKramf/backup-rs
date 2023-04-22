import { Component, EventEmitter, Input, Output } from '@angular/core';
import { ProfileConfig } from '../profile-config';
import { ApiServiceService } from '../api-service.service';

@Component({
  selector: 'app-config-editor',
  templateUrl: './config-editor.component.html',
  styleUrls: ['./config-editor.component.scss']
})
export class ConfigEditorComponent {
  @Input() profileConfig!: ProfileConfig;
  @Output() reload = new EventEmitter<boolean>;

  constructor(private readonly api: ApiServiceService) {}

  store(): void {
    this.api.updateProfileConfigs(this.profileConfig)
      .subscribe(config => {
        console.log(config)
        this.reload.emit(true);
      });
  }

  delete(): void {
    this.api.deleteProfileConfig(this.profileConfig.uuid).subscribe(() => {
      this.reload.emit(true);
    });
  }
}
