import { Component, OnInit } from '@angular/core';
import { ApiServiceService } from './api-service.service';
import { ProfileConfig } from './profile-config';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss']
})
export class AppComponent implements OnInit {
  title = 'frontend';
  profileConfigs: ProfileConfig[] = [];

  constructor(private api: ApiServiceService) {}

  ngOnInit(): void {
    this.api.getProfileConfigs()
      .subscribe(configs => {
        console.log(configs);
        this.profileConfigs = configs;
      });
  }
}
