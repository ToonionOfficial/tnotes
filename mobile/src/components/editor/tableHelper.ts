import type { EditorBridge } from "@10play/tentap-editor"

export const TABLE_INIT_SCRIPT = `
(function() {
  if (window.tnotesTable) return;

  window.tnotesTable = {
    insertTable: function(rows, cols) {
      rows = rows || 3;
      cols = cols || 3;
      var html = '<table class="tnotes-table"><thead><tr>';
      for (var c = 0; c < cols; c++) {
        html += '<th>Header ' + (c + 1) + '</th>';
      }
      html += '</tr></thead><tbody>';
      for (var r = 0; r < rows - 1; r++) {
        html += '<tr>';
        for (var c = 0; c < cols; c++) {
          html += '<td>Cell</td>';
        }
        html += '</tr>';
      }
      html += '</tbody></table><p></p>';
      document.execCommand('insertHTML', false, html);
    },

    addRowBelow: function() {
      var sel = window.getSelection();
      if (!sel || !sel.anchorNode) return;
      var cell = sel.anchorNode.nodeType === 1 ? sel.anchorNode.closest('td, th') : sel.anchorNode.parentElement ? sel.anchorNode.parentElement.closest('td, th') : null;
      if (!cell) return;
      var row = cell.closest('tr');
      if (!row) return;
      var colCount = row.children.length;
      var newRow = document.createElement('tr');
      for (var i = 0; i < colCount; i++) {
        var td = document.createElement('td');
        td.innerHTML = '<br>';
        newRow.appendChild(td);
      }
      row.parentNode && row.parentNode.insertBefore(newRow, row.nextSibling);
    },

    addRowAbove: function() {
      var sel = window.getSelection();
      if (!sel || !sel.anchorNode) return;
      var cell = sel.anchorNode.nodeType === 1 ? sel.anchorNode.closest('td, th') : sel.anchorNode.parentElement ? sel.anchorNode.parentElement.closest('td, th') : null;
      if (!cell) return;
      var row = cell.closest('tr');
      if (!row) return;
      var colCount = row.children.length;
      var newRow = document.createElement('tr');
      for (var i = 0; i < colCount; i++) {
        var td = document.createElement('td');
        td.innerHTML = '<br>';
        newRow.appendChild(td);
      }
      row.parentNode && row.parentNode.insertBefore(newRow, row);
    },

    addColumnRight: function() {
      var sel = window.getSelection();
      if (!sel || !sel.anchorNode) return;
      var cell = sel.anchorNode.nodeType === 1 ? sel.anchorNode.closest('td, th') : sel.anchorNode.parentElement ? sel.anchorNode.parentElement.closest('td, th') : null;
      if (!cell) return;
      var table = cell.closest('table');
      if (!table) return;
      var colIndex = Array.from(cell.parentElement ? cell.parentElement.children : []).indexOf(cell);
      if (colIndex === -1) return;
      var rows = table.querySelectorAll('tr');
      rows.forEach(function(r, idx) {
        var newCell = idx === 0 && r.querySelector('th') ? document.createElement('th') : document.createElement('td');
        newCell.innerHTML = '<br>';
        var targetChild = r.children[colIndex];
        if (targetChild && targetChild.nextSibling) {
          r.insertBefore(newCell, targetChild.nextSibling);
        } else {
          r.appendChild(newCell);
        }
      });
    },

    addColumnLeft: function() {
      var sel = window.getSelection();
      if (!sel || !sel.anchorNode) return;
      var cell = sel.anchorNode.nodeType === 1 ? sel.anchorNode.closest('td, th') : sel.anchorNode.parentElement ? sel.anchorNode.parentElement.closest('td, th') : null;
      if (!cell) return;
      var table = cell.closest('table');
      if (!table) return;
      var colIndex = Array.from(cell.parentElement ? cell.parentElement.children : []).indexOf(cell);
      if (colIndex === -1) return;
      var rows = table.querySelectorAll('tr');
      rows.forEach(function(r, idx) {
        var newCell = idx === 0 && r.querySelector('th') ? document.createElement('th') : document.createElement('td');
        newCell.innerHTML = '<br>';
        var targetChild = r.children[colIndex];
        if (targetChild) {
          r.insertBefore(newCell, targetChild);
        } else {
          r.appendChild(newCell);
        }
      });
    },

    deleteRow: function() {
      var sel = window.getSelection();
      if (!sel || !sel.anchorNode) return;
      var cell = sel.anchorNode.nodeType === 1 ? sel.anchorNode.closest('td, th') : sel.anchorNode.parentElement ? sel.anchorNode.parentElement.closest('td, th') : null;
      if (!cell) return;
      var row = cell.closest('tr');
      var table = cell.closest('table');
      if (row && table) {
        if (table.querySelectorAll('tr').length <= 1) {
          table.remove();
        } else {
          row.remove();
        }
      }
    },

    deleteColumn: function() {
      var sel = window.getSelection();
      if (!sel || !sel.anchorNode) return;
      var cell = sel.anchorNode.nodeType === 1 ? sel.anchorNode.closest('td, th') : sel.anchorNode.parentElement ? sel.anchorNode.parentElement.closest('td, th') : null;
      if (!cell) return;
      var table = cell.closest('table');
      if (!table) return;
      var colIndex = Array.from(cell.parentElement ? cell.parentElement.children : []).indexOf(cell);
      if (colIndex === -1) return;
      var rows = table.querySelectorAll('tr');
      if (cell.parentElement && cell.parentElement.children.length <= 1) {
        table.remove();
        return;
      }
      rows.forEach(function(r) {
        if (r.children[colIndex]) {
          r.children[colIndex].remove();
        }
      });
    },

    deleteTable: function() {
      var sel = window.getSelection();
      if (!sel || !sel.anchorNode) return;
      var cell = sel.anchorNode.nodeType === 1 ? sel.anchorNode.closest('td, th') : sel.anchorNode.parentElement ? sel.anchorNode.parentElement.closest('td, th') : null;
      var table = cell ? cell.closest('table') : null;
      if (table) {
        table.remove();
      }
    }
  };
})();
`

export function insertTable(editor: EditorBridge, rows = 3, cols = 3) {
  editor.injectJS(`
    ${TABLE_INIT_SCRIPT}
    window.tnotesTable.insertTable(${rows}, ${cols});
  `)
}

export function addTableRowBelow(editor: EditorBridge) {
  editor.injectJS(`
    ${TABLE_INIT_SCRIPT}
    window.tnotesTable.addRowBelow();
  `)
}

export function addTableRowAbove(editor: EditorBridge) {
  editor.injectJS(`
    ${TABLE_INIT_SCRIPT}
    window.tnotesTable.addRowAbove();
  `)
}

export function addTableColumnRight(editor: EditorBridge) {
  editor.injectJS(`
    ${TABLE_INIT_SCRIPT}
    window.tnotesTable.addColumnRight();
  `)
}

export function addTableColumnLeft(editor: EditorBridge) {
  editor.injectJS(`
    ${TABLE_INIT_SCRIPT}
    window.tnotesTable.addColumnLeft();
  `)
}

export function deleteTableRow(editor: EditorBridge) {
  editor.injectJS(`
    ${TABLE_INIT_SCRIPT}
    window.tnotesTable.deleteRow();
  `)
}

export function deleteTableColumn(editor: EditorBridge) {
  editor.injectJS(`
    ${TABLE_INIT_SCRIPT}
    window.tnotesTable.deleteColumn();
  `)
}

export function deleteTable(editor: EditorBridge) {
  editor.injectJS(`
    ${TABLE_INIT_SCRIPT}
    window.tnotesTable.deleteTable();
  `)
}
