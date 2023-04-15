import { Component, Input } from '@angular/core';

@Component({
  selector: 'app-text-list',
  templateUrl: './text-list.component.html',
  styleUrls: ['./text-list.component.scss']
})
export class TextListComponent {
  @Input() list: string[] = [];
  @Input() placeholder = '';

  addElement(): void {
    this.list.push('');
  }

  removeElement(index: number): void {
    this.list.splice(index, 1)
  }
}
