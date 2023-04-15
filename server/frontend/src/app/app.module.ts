import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';
import { HttpClientModule } from '@angular/common/http';
import { FormsModule } from '@angular/forms';

import { AppComponent } from './app.component';
import { SideNavComponent } from './side-nav/side-nav.component';
import { ConfigEditorComponent } from './config-editor/config-editor.component';
import { DashboardComponent } from './dashboard/dashboard.component';
import { ConfigInfoComponent } from './config-editor/config-info/config-info.component';
import { FileSelectionComponent } from './config-editor/file-selection/file-selection.component';
import { TextListComponent } from './config-editor/text-list/text-list.component';

@NgModule({
  declarations: [
    AppComponent,
    SideNavComponent,
    ConfigEditorComponent,
    DashboardComponent,
    ConfigInfoComponent,
    FileSelectionComponent,
    TextListComponent
  ],
  imports: [
    BrowserModule,
    FormsModule,
    HttpClientModule
  ],
  providers: [],
  bootstrap: [AppComponent]
})
export class AppModule { }
