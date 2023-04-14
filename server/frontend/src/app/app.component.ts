import { Component, OnInit, ViewChild } from '@angular/core';
import { ApiServiceService } from './api-service.service';
import { ProfileConfig } from './profile-config';
import { SideNavComponent } from './side-nav/side-nav.component';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss']
})
export class AppComponent implements OnInit {

  @ViewChild(SideNavComponent)
  private sideNav!: SideNavComponent;

  title = 'Backupper';
  profileConfigs: ProfileConfig[] = [];
  selected?: ProfileConfig;

  constructor(private api: ApiServiceService) {}

  ngOnInit(): void {
    this.getProfileConfigs();
  }

  getProfileConfigs(): void {
    this.api.getProfileConfigs()
      .subscribe(configs => {
        console.log(configs);
        this.profileConfigs = configs.sort((a, b) => a.name.localeCompare(b.name));
      });
  }

  onSelect(choice?: ProfileConfig): void {
    this.selected = choice;
    this.sideNav.selectedUuid = this.selected?.uuid ?? '';
  }
}
