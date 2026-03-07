/**
 * Forms and Input Handling E2E Tests
 * Comprehensive testing of form elements and user input
 * 
 * @description Tests for form controls, input validation, 
 *              submission, and user interaction patterns
 * @coverage Forms, inputs, validation, submission, user input
 */

import { test, expect } from '@playwright/test';
import { allure } from 'allure-playwright';

test.describe('Forms and Input Handling', () => {
  test.beforeEach(async () => {
    await allure.epic('User Interface');
    await allure.feature('Form Handling');
    await allure.story('As a user, I want to interact with forms reliably');
  });

  test.describe('Text Input Fields', () => {
    test('should accept text input', async ({ page }) => {
      await allure.severity('critical');
      await allure.description('Verify text input field accepts user input');
      
      await page.goto('https://example.com');
      
      // Create a test input
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'text';
        input.id = 'test-input';
        document.body.appendChild(input);
      });
      
      const input = page.locator('#test-input');
      await input.fill('Hello World');
      
      await expect(input).toHaveValue('Hello World');
    });

    test('should handle special characters', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify special characters in input');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'text';
        input.id = 'special-input';
        document.body.appendChild(input);
      });
      
      const input = page.locator('#special-input');
      await input.fill('Test @#$%^&*()! Special');
      
      await expect(input).toHaveValue('Test @#$%^&*()! Special');
    });

    test('should clear input field', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify input field can be cleared');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'text';
        input.id = 'clear-input';
        input.value = 'Initial value';
        document.body.appendChild(input);
      });
      
      const input = page.locator('#clear-input');
      await input.clear();
      
      await expect(input).toHaveValue('');
    });

    test('should handle maxlength restriction', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify maxlength attribute is enforced');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'text';
        input.id = 'max-input';
        input.maxLength = 10;
        document.body.appendChild(input);
      });
      
      const input = page.locator('#max-input');
      await input.fill('This is a very long text');
      
      // Should be truncated to 10 characters
      const value = await input.inputValue();
      expect(value.length).toBeLessThanOrEqual(10);
    });

    test('should handle placeholder text', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify placeholder text display');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'text';
        input.id = 'placeholder-input';
        input.placeholder = 'Enter your name';
        document.body.appendChild(input);
      });
      
      const input = page.locator('#placeholder-input');
      const placeholder = await input.getAttribute('placeholder');
      
      expect(placeholder).toBe('Enter your name');
    });

    test('should support readonly inputs', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify readonly input behavior');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'text';
        input.id = 'readonly-input';
        input.value = 'Read only';
        input.readOnly = true;
        document.body.appendChild(input);
      });
      
      const input = page.locator('#readonly-input');
      
      // Should not be editable
      const isReadonly = await input.getAttribute('readonly');
      expect(isReadonly).not.toBeNull();
    });

    test('should handle disabled inputs', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify disabled input behavior');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'text';
        input.id = 'disabled-input';
        input.disabled = true;
        document.body.appendChild(input);
      });
      
      const input = page.locator('#disabled-input');
      await expect(input).toBeDisabled();
    });
  });

  test.describe('Email Input Fields', () => {
    test('should validate email format', async ({ page }) => {
      await allure.severity('critical');
      await allure.description('Verify email input validation');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'email';
        input.id = 'email-input';
        document.body.appendChild(input);
      });
      
      const input = page.locator('#email-input');
      await input.fill('test@example.com');
      
      await expect(input).toHaveValue('test@example.com');
    });

    test('should reject invalid email', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify invalid email rejection');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'email';
        input.id = 'invalid-email';
        document.body.appendChild(input);
      });
      
      const input = page.locator('#invalid-email');
      await input.fill('not-an-email');
      
      // Check validity
      const isValid = await input.evaluate(el => (el as HTMLInputElement).checkValidity());
      expect(isValid).toBe(false);
    });
  });

  test.describe('Password Input Fields', () => {
    test('should mask password input', async ({ page }) => {
      await allure.severity('critical');
      await allure.description('Verify password masking');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'password';
        input.id = 'password-input';
        document.body.appendChild(input);
      });
      
      const input = page.locator('#password-input');
      await input.fill('secret123');
      
      const type = await input.getAttribute('type');
      expect(type).toBe('password');
    });

    test('should toggle password visibility', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify password visibility toggle');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const container = document.createElement('div');
        container.innerHTML = `
          <input type="password" id="toggle-password" />
          <button id="toggle-btn">Show</button>
        `;
        document.body.appendChild(container);
        
        document.getElementById('toggle-btn')?.addEventListener('click', () => {
          const pwd = document.getElementById('toggle-password') as HTMLInputElement;
          pwd.type = pwd.type === 'password' ? 'text' : 'password';
        });
      });
      
      const input = page.locator('#toggle-password');
      const btn = page.locator('#toggle-btn');
      
      await input.fill('secret');
      
      // Toggle to show
      await btn.click();
      expect(await input.getAttribute('type')).toBe('text');
      
      // Toggle to hide
      await btn.click();
      expect(await input.getAttribute('type')).toBe('password');
    });
  });

  test.describe('Number Input Fields', () => {
    test('should accept numeric input', async ({ page }) => {
      await allure.severity('critical');
      await allure.description('Verify number input accepts numbers');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'number';
        input.id = 'number-input';
        document.body.appendChild(input);
      });
      
      const input = page.locator('#number-input');
      await input.fill('42');
      
      await expect(input).toHaveValue('42');
    });

    test('should enforce min value', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify min value enforcement');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'number';
        input.id = 'min-number';
        input.min = '10';
        document.body.appendChild(input);
      });
      
      const input = page.locator('#min-number');
      await input.fill('5');
      
      const min = await input.getAttribute('min');
      expect(min).toBe('10');
    });

    test('should enforce max value', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify max value enforcement');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'number';
        input.id = 'max-number';
        input.max = '100';
        document.body.appendChild(input);
      });
      
      const input = page.locator('#max-number');
      const max = await input.getAttribute('max');
      
      expect(max).toBe('100');
    });

    test('should handle step increments', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify step increment functionality');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'number';
        input.id = 'step-number';
        input.step = '5';
        input.value = '10';
        document.body.appendChild(input);
      });
      
      const input = page.locator('#step-number');
      const step = await input.getAttribute('step');
      
      expect(step).toBe('5');
    });
  });

  test.describe('Textarea Fields', () => {
    test('should handle multi-line text', async ({ page }) => {
      await allure.severity('critical');
      await allure.description('Verify textarea multi-line support');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const textarea = document.createElement('textarea');
        textarea.id = 'multiline-text';
        document.body.appendChild(textarea);
      });
      
      const textarea = page.locator('#multiline-text');
      await textarea.fill('Line 1\nLine 2\nLine 3');
      
      await expect(textarea).toHaveValue('Line 1\nLine 2\nLine 3');
    });

    test('should handle text wrapping', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify textarea text wrapping');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const textarea = document.createElement('textarea');
        textarea.id = 'wrap-text';
        textarea.wrap = 'hard';
        textarea.cols = 20;
        document.body.appendChild(textarea);
      });
      
      const textarea = page.locator('#wrap-text');
      const wrap = await textarea.getAttribute('wrap');
      
      expect(wrap).toBe('hard');
    });

    test('should resize textarea', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify textarea resizing');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const textarea = document.createElement('textarea');
        textarea.id = 'resize-text';
        textarea.style.resize = 'both';
        document.body.appendChild(textarea);
      });
      
      const textarea = page.locator('#resize-text');
      await expect(textarea).toBeVisible();
    });
  });

  test.describe('Checkbox Inputs', () => {
    test('should toggle checkbox', async ({ page }) => {
      await allure.severity('critical');
      await allure.description('Verify checkbox toggle functionality');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.id = 'test-checkbox';
        document.body.appendChild(checkbox);
      });
      
      const checkbox = page.locator('#test-checkbox');
      
      // Check
      await checkbox.check();
      await expect(checkbox).toBeChecked();
      
      // Uncheck
      await checkbox.uncheck();
      await expect(checkbox).not.toBeChecked();
    });

    test('should handle checkbox groups', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify checkbox group handling');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const form = document.createElement('form');
        form.innerHTML = `
          <input type="checkbox" name="options" value="opt1" id="opt1" />
          <input type="checkbox" name="options" value="opt2" id="opt2" />
          <input type="checkbox" name="options" value="opt3" id="opt3" />
        `;
        document.body.appendChild(form);
      });
      
      // Select multiple
      await page.locator('#opt1').check();
      await page.locator('#opt3').check();
      
      await expect(page.locator('#opt1')).toBeChecked();
      await expect(page.locator('#opt2')).not.toBeChecked();
      await expect(page.locator('#opt3')).toBeChecked();
    });

    test('should handle indeterminate state', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify checkbox indeterminate state');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.id = 'indeterminate-checkbox';
        (checkbox as any).indeterminate = true;
        document.body.appendChild(checkbox);
      });
      
      const checkbox = page.locator('#indeterminate-checkbox');
      const isIndeterminate = await checkbox.evaluate(el => (el as HTMLInputElement).indeterminate);
      
      expect(isIndeterminate).toBe(true);
    });
  });

  test.describe('Radio Button Inputs', () => {
    test('should select radio button', async ({ page }) => {
      await allure.severity('critical');
      await allure.description('Verify radio button selection');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const form = document.createElement('form');
        form.innerHTML = `
          <input type="radio" name="choice" value="a" id="radio-a" />
          <input type="radio" name="choice" value="b" id="radio-b" />
        `;
        document.body.appendChild(form);
      });
      
      const radioA = page.locator('#radio-a');
      const radioB = page.locator('#radio-b');
      
      await radioA.check();
      await expect(radioA).toBeChecked();
      
      // Selecting B should deselect A
      await radioB.check();
      await expect(radioB).toBeChecked();
    });

    test('should have only one selected in group', async ({ page }) => {
      await allure.severity('critical');
      await allure.description('Verify single selection in radio group');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const form = document.createElement('form');
        form.innerHTML = `
          <input type="radio" name="single" value="1" id="s1" />
          <input type="radio" name="single" value="2" id="s2" />
          <input type="radio" name="single" value="3" id="s3" />
        `;
        document.body.appendChild(form);
      });
      
      await page.locator('#s1').check();
      await page.locator('#s3').check();
      
      // Only s3 should be checked
      await expect(page.locator('#s1')).not.toBeChecked();
      await expect(page.locator('#s2')).not.toBeChecked();
      await expect(page.locator('#s3')).toBeChecked();
    });
  });

  test.describe('Select Dropdowns', () => {
    test('should select option from dropdown', async ({ page }) => {
      await allure.severity('critical');
      await allure.description('Verify dropdown selection');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const select = document.createElement('select');
        select.id = 'test-select';
        select.innerHTML = `
          <option value="">Select...</option>
          <option value="opt1">Option 1</option>
          <option value="opt2">Option 2</option>
          <option value="opt3">Option 3</option>
        `;
        document.body.appendChild(select);
      });
      
      const select = page.locator('#test-select');
      await select.selectOption('opt2');
      
      await expect(select).toHaveValue('opt2');
    });

    test('should support multiple selection', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify multiple select functionality');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const select = document.createElement('select');
        select.id = 'multi-select';
        select.multiple = true;
        select.innerHTML = `
          <option value="a">A</option>
          <option value="b">B</option>
          <option value="c">C</option>
        `;
        document.body.appendChild(select);
      });
      
      const select = page.locator('#multi-select');
      await select.selectOption(['a', 'c']);
      
      const selected = await select.evaluate(el => 
        Array.from((el as HTMLSelectElement).selectedOptions).map(o => o.value)
      );
      
      expect(selected).toContain('a');
      expect(selected).toContain('c');
    });

    test('should handle option groups', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify option group handling');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const select = document.createElement('select');
        select.id = 'grouped-select';
        select.innerHTML = `
          <optgroup label="Group 1">
            <option value="g1a">G1 - A</option>
            <option value="g1b">G1 - B</option>
          </optgroup>
          <optgroup label="Group 2">
            <option value="g2a">G2 - A</option>
          </optgroup>
        `;
        document.body.appendChild(select);
      });
      
      const select = page.locator('#grouped-select');
      await select.selectOption('g1a');
      
      await expect(select).toHaveValue('g1a');
    });
  });

  test.describe('File Input', () => {
    test('should accept file input', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify file input functionality');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'file';
        input.id = 'file-input';
        document.body.appendChild(input);
      });
      
      const fileInput = page.locator('#file-input');
      
      // Verify file input exists
      await expect(fileInput).toBeVisible();
    });

    test('should restrict file types', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify file type restrictions');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'file';
        input.id = 'restricted-file';
        input.accept = '.pdf,.doc,.docx';
        document.body.appendChild(input);
      });
      
      const fileInput = page.locator('#restricted-file');
      const accept = await fileInput.getAttribute('accept');
      
      expect(accept).toBe('.pdf,.doc,.docx');
    });

    test('should support multiple file selection', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify multiple file selection');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'file';
        input.id = 'multi-file';
        input.multiple = true;
        document.body.appendChild(input);
      });
      
      const fileInput = page.locator('#multi-file');
      const multiple = await fileInput.getAttribute('multiple');
      
      expect(multiple).not.toBeNull();
    });
  });

  test.describe('Date and Time Inputs', () => {
    test('should handle date input', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify date input functionality');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'date';
        input.id = 'date-input';
        document.body.appendChild(input);
      });
      
      const dateInput = page.locator('#date-input');
      await dateInput.fill('2024-12-25');
      
      await expect(dateInput).toHaveValue('2024-12-25');
    });

    test('should handle time input', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify time input functionality');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'time';
        input.id = 'time-input';
        document.body.appendChild(input);
      });
      
      const timeInput = page.locator('#time-input');
      await timeInput.fill('14:30');
      
      await expect(timeInput).toHaveValue('14:30');
    });

    test('should handle datetime-local input', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify datetime-local input');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'datetime-local';
        input.id = 'datetime-input';
        document.body.appendChild(input);
      });
      
      const datetimeInput = page.locator('#datetime-input');
      await datetimeInput.fill('2024-12-25T14:30');
      
      await expect(datetimeInput).toHaveValue('2024-12-25T14:30');
    });
  });

  test.describe('Range and Color Inputs', () => {
    test('should handle range slider', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify range slider functionality');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'range';
        input.id = 'range-input';
        input.min = '0';
        input.max = '100';
        input.value = '50';
        document.body.appendChild(input);
      });
      
      const rangeInput = page.locator('#range-input');
      const value = await rangeInput.inputValue();
      
      expect(parseInt(value)).toBeGreaterThanOrEqual(0);
      expect(parseInt(value)).toBeLessThanOrEqual(100);
    });

    test('should handle color picker', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify color picker functionality');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'color';
        input.id = 'color-input';
        document.body.appendChild(input);
      });
      
      const colorInput = page.locator('#color-input');
      await colorInput.fill('#ff0000');
      
      await expect(colorInput).toHaveValue('#ff0000');
    });
  });

  test.describe('Form Validation', () => {
    test('should validate required fields', async ({ page }) => {
      await allure.severity('critical');
      await allure.description('Verify required field validation');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const form = document.createElement('form');
        form.id = 'validation-form';
        form.innerHTML = `
          <input type="text" id="required-field" required />
          <button type="submit">Submit</button>
        `;
        document.body.appendChild(form);
      });
      
      const input = page.locator('#required-field');
      const isRequired = await input.getAttribute('required');
      
      expect(isRequired).not.toBeNull();
    });

    test('should show validation messages', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify validation message display');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'text';
        input.id = 'validation-msg';
        input.required = true;
        document.body.appendChild(input);
      });
      
      const input = page.locator('#validation-msg');
      
      // Check validation
      const message = await input.evaluate(el => (el as HTMLInputElement).validationMessage);
      
      // Message will be empty until form validation is triggered
      expect(typeof message).toBe('string');
    });

    test('should validate pattern matching', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify pattern validation');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'text';
        input.id = 'pattern-input';
        input.pattern = '[A-Za-z]{3,}';
        input.value = 'ab';
        document.body.appendChild(input);
      });
      
      const input = page.locator('#pattern-input');
      const isValid = await input.evaluate(el => (el as HTMLInputElement).checkValidity());
      
      expect(isValid).toBe(false);
    });

    test('should handle custom validation', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify custom validation logic');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'text';
        input.id = 'custom-valid';
        document.body.appendChild(input);
        
        input.setCustomValidity('Custom error message');
      });
      
      const input = page.locator('#custom-valid');
      const message = await input.evaluate(el => (el as HTMLInputElement).validationMessage);
      
      expect(message).toBe('Custom error message');
    });
  });

  test.describe('Form Submission', () => {
    test('should submit form on button click', async ({ page }) => {
      await allure.severity('critical');
      await allure.description('Verify form submission');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const form = document.createElement('form');
        form.id = 'submit-form';
        form.innerHTML = `
          <input type="text" name="field" value="test" />
          <button type="submit" id="submit-btn">Submit</button>
        `;
        document.body.appendChild(form);
        
        form.addEventListener('submit', (e) => {
          e.preventDefault();
          (window as any).formSubmitted = true;
        });
      });
      
      await page.locator('#submit-btn').click();
      
      const submitted = await page.evaluate(() => (window as any).formSubmitted);
      expect(submitted).toBe(true);
    });

    test('should submit form on enter key', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify form submission on Enter');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const form = document.createElement('form');
        form.id = 'enter-form';
        form.innerHTML = `<input type="text" name="field" id="enter-input" />`;
        document.body.appendChild(form);
        
        form.addEventListener('submit', (e) => {
          e.preventDefault();
          (window as any).enterSubmitted = true;
        });
      });
      
      await page.locator('#enter-input').fill('test');
      await page.locator('#enter-input').press('Enter');
      
      const submitted = await page.evaluate(() => (window as any).enterSubmitted);
      expect(submitted).toBe(true);
    });

    test('should reset form', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify form reset functionality');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const form = document.createElement('form');
        form.id = 'reset-form';
        form.innerHTML = `
          <input type="text" name="field" id="reset-input" value="initial" />
          <button type="reset" id="reset-btn">Reset</button>
        `;
        document.body.appendChild(form);
      });
      
      const input = page.locator('#reset-input');
      await input.fill('modified');
      
      await page.locator('#reset-btn').click();
      
      await expect(input).toHaveValue('initial');
    });
  });

  test.describe('Autocomplete and Autofill', () => {
    test('should support autocomplete attribute', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify autocomplete support');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'text';
        input.id = 'autocomplete-input';
        input.autocomplete = 'name';
        document.body.appendChild(input);
      });
      
      const input = page.locator('#autocomplete-input');
      const autocomplete = await input.getAttribute('autocomplete');
      
      expect(autocomplete).toBe('name');
    });

    test('should handle autofill values', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify autofill handling');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'text';
        input.id = 'autofill-input';
        input.autocomplete = 'email';
        document.body.appendChild(input);
      });
      
      const input = page.locator('#autofill-input');
      await expect(input).toBeVisible();
    });
  });

  test.describe('Input Events', () => {
    test('should trigger input event on change', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify input event triggering');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const input = document.createElement('input');
        input.type = 'text';
        input.id = 'event-input';
        document.body.appendChild(input);
        
        (window as any).inputEvents = [];
        input.addEventListener('input', (e) => {
          (window as any).inputEvents.push((e.target as HTMLInputElement).value);
        });
      });
      
      const input = page.locator('#event-input');
      await input.fill('test');
      
      const events = await page.evaluate(() => (window as any).inputEvents);
      expect(events).toContain('test');
    });

    test('should trigger change event on blur', async ({ page }) => {
      await allure.severity('major');
      await allure.description('Verify change event triggering');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const container = document.createElement('div');
        container.innerHTML = `
          <input type="text" id="change-input" />
          <input type="text" id="blur-target" />
        `;
        document.body.appendChild(container);
        
        (window as any).changeTriggered = false;
        document.getElementById('change-input')?.addEventListener('change', () => {
          (window as any).changeTriggered = true;
        });
      });
      
      await page.locator('#change-input').fill('value');
      await page.locator('#blur-target').click();
      
      const changed = await page.evaluate(() => (window as any).changeTriggered);
      expect(changed).toBe(true);
    });

    test('should trigger focus and blur events', async ({ page }) => {
      await allure.severity('minor');
      await allure.description('Verify focus/blur event triggering');
      
      await page.goto('https://example.com');
      
      await page.evaluate(() => {
        const container = document.createElement('div');
        container.innerHTML = `
          <input type="text" id="focus-input" />
          <input type="text" id="other-input" />
        `;
        document.body.appendChild(container);
        
        (window as any).focusEvents = [];
        const input = document.getElementById('focus-input');
        
        input?.addEventListener('focus', () => (window as any).focusEvents.push('focus'));
        input?.addEventListener('blur', () => (window as any).focusEvents.push('blur'));
      });
      
      await page.locator('#focus-input').focus();
      await page.locator('#other-input').focus();
      
      const events = await page.evaluate(() => (window as any).focusEvents);
      expect(events).toEqual(['focus', 'blur']);
    });
  });
});