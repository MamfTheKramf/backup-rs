import { Component, Input } from '@angular/core';

@Component({
  selector: 'app-number-list',
  templateUrl: './number-list.component.html',
  styleUrls: ['./number-list.component.scss']
})
export class NumberListComponent {
  @Input() list: number[] = [];
  @Input() placeholder = '';
  @Input() minimum!: number;
  @Input() maximum!: number;
  @Input() offset!: number;

  addElement(): void {
    console.log('Add element');
    console.log(this.list);
    this.list.push(0);
    console.log(this.list);
  }

  removeElement(index: number): void {
    this.list.splice(index, 1)
  }

  /**
   * Updates the given list element by taking the value in the `event`-target and subtracting `offset`
   * @param event 
   * @param index 
   */
  updateElement(event: Event, index: number): void {
    const newValue = Number((event.target as HTMLInputElement).value);
    if (newValue < this.minimum || newValue > this.maximum) {
      return;
    }
    if (index >= 0 && index < this.list.length) {
      this.list[index] = newValue - this.offset;
    }
  }

  getElement(index: number): number {
    return this.list[index] + this.offset;
  }
}
