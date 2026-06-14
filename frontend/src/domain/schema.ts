import Ajv2020, { type ErrorObject } from "ajv/dist/2020";
import addFormats from "ajv-formats";

import type { JsonSchema, ParameterDraft, ToolDraft, ToolParameter, ValidationRules } from "./types";

const validator = createValidator();

export interface ValidationResult {
  valid: boolean;
  errors: Record<string, string>;
  data: Record<string, unknown>;
  schema: JsonSchema;
}

export function buildInputSchema(tool: Pick<ToolDraft, "parameters" | "working_dir_expose" | "working_dir_required">): JsonSchema {
  const properties = buildProperties(tool.parameters);
  const required = collectRequiredNames(tool.parameters);

  addWorkingDirectorySchema(properties, required, tool);

  return {
    $schema: "https://json-schema.org/draft/2020-12/schema",
    type: "object",
    properties,
    required,
    additionalProperties: false,
  };
}

export function validateFormData(tool: ToolDraft, formData: Record<string, unknown>): ValidationResult {
  const schema = buildInputSchema(tool);
  const data = { ...formData };
  const validate = validator.compile(schema);
  const valid = validate(data);

  return {
    valid,
    errors: valid ? {} : collectValidationErrors(validate.errors ?? []),
    data,
    schema,
  };
}

export function createParamsFromDraftValues(
  parameters: ParameterDraft[],
  values: Record<string, unknown>,
): Record<string, unknown> {
  return parameters.reduce<Record<string, unknown>>((params, parameter) => {
    const value = normalizeValueForParameter(parameter, values[parameter.name]);
    if (value !== undefined) {
      params[parameter.name] = value;
    }

    return params;
  }, {});
}

export function normalizeValueForParameter(parameter: ParameterDraft, value: unknown): unknown {
  if (parameter.type === "boolean") {
    return value === true || value === "true";
  }

  if (value == null || String(value).trim() === "") {
    return undefined;
  }

  if (parameter.type === "number") {
    return Number(value);
  }

  return String(value);
}

export function draftParameterToPayload(parameter: ParameterDraft): Omit<ToolParameter, "id" | "tool_id"> {
  return {
    name: parameter.name,
    type: parameter.type,
    description: emptyToNull(parameter.description),
    required: parameter.required,
    default_value: normalizeDefaultValue(parameter),
    validation: normalizeValidation(parameter.validation),
    position: parameter.position,
  };
}

function createValidator(): Ajv2020 {
  const ajv = new Ajv2020({ allErrors: true, coerceTypes: true });
  addFormats(ajv);
  ajv.addFormat("path", true);
  return ajv;
}

function buildProperties(parameters: ParameterDraft[]): Record<string, unknown> {
  return parameters.reduce<Record<string, unknown>>((properties, parameter) => {
    if (parameter.name.trim()) {
      properties[parameter.name] = parameterSchema(parameter);
    }

    return properties;
  }, {});
}

function collectRequiredNames(parameters: ParameterDraft[]): string[] {
  return parameters.filter(isRequiredNamedParameter).map((parameter) => parameter.name);
}

function isRequiredNamedParameter(parameter: ParameterDraft): boolean {
  return parameter.required && parameter.name.trim().length > 0;
}

function addWorkingDirectorySchema(
  properties: Record<string, unknown>,
  required: string[],
  tool: Pick<ToolDraft, "working_dir_expose" | "working_dir_required">,
): void {
  if (!tool.working_dir_expose) {
    return;
  }

  properties.working_dir = {
    type: "string",
    format: "path",
    description: "Working directory used to execute the tool",
  };

  if (tool.working_dir_required) {
    required.push("working_dir");
  }
}

function parameterSchema(parameter: ParameterDraft): Record<string, unknown> {
  return cleanObject({
    type: jsonSchemaType(parameter),
    description: emptyToUndefined(parameter.description),
    default: normalizeDefaultValue(parameter),
    minimum: numericMinimum(parameter),
    maximum: numericMaximum(parameter),
    minLength: textMinimum(parameter),
    maxLength: textMaximum(parameter),
    pattern: parameter.validation.regex || undefined,
    format: parameterFormat(parameter),
    enum: enumValues(parameter),
  });
}

function jsonSchemaType(parameter: ParameterDraft): "string" | "number" | "integer" | "boolean" {
  if (parameter.type === "boolean") {
    return "boolean";
  }

  if (parameter.type === "number") {
    return parameter.validation.integer ? "integer" : "number";
  }

  return "string";
}

function numericMinimum(parameter: ParameterDraft): number | undefined {
  return parameter.type === "number" ? nullableNumber(parameter.validation.min) : undefined;
}

function numericMaximum(parameter: ParameterDraft): number | undefined {
  return parameter.type === "number" ? nullableNumber(parameter.validation.max) : undefined;
}

function textMinimum(parameter: ParameterDraft): number | undefined {
  return isTextParameter(parameter) ? lengthLimit(parameter.validation.min) : undefined;
}

function textMaximum(parameter: ParameterDraft): number | undefined {
  return isTextParameter(parameter) ? lengthLimit(parameter.validation.max) : undefined;
}

function parameterFormat(parameter: ParameterDraft): string | undefined {
  if (parameter.type === "path") {
    return "path";
  }

  return parameter.validation.format || undefined;
}

function enumValues(parameter: ParameterDraft): string[] | undefined {
  if (parameter.type !== "enum" && parameter.validation.enum_values.length === 0) {
    return undefined;
  }

  return parameter.validation.enum_values;
}

function isTextParameter(parameter: ParameterDraft): boolean {
  return parameter.type === "string" || parameter.type === "path";
}

function lengthLimit(value?: number | null): number | undefined {
  const number = nullableNumber(value);
  return number == null || number < 0 ? undefined : Math.round(number);
}

function nullableNumber(value?: number | null): number | undefined {
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function normalizeDefaultValue(parameter: ParameterDraft): unknown {
  const value = parameter.default_value;
  if (value == null || value === "") {
    return undefined;
  }

  if (parameter.type === "number") {
    return Number.isNaN(Number(value)) ? undefined : Number(value);
  }

  if (parameter.type === "boolean") {
    return value === true || value === "true";
  }

  return String(value);
}

function normalizeValidation(validation: ValidationRules): ValidationRules {
  return {
    min: nullableNumber(validation.min) ?? null,
    max: nullableNumber(validation.max) ?? null,
    regex: emptyToNull(validation.regex),
    format: emptyToNull(validation.format),
    integer: validation.integer,
    enum_values: dedupeText(validation.enum_values),
  };
}

function dedupeText(values: string[]): string[] {
  return values.reduce<string[]>((result, value) => {
    const normalized = value.trim();
    if (normalized && !result.some((existing) => existing.toLowerCase() === normalized.toLowerCase())) {
      result.push(normalized);
    }

    return result;
  }, []);
}

function collectValidationErrors(errors: ErrorObject[]): Record<string, string> {
  return errors.reduce<Record<string, string>>((result, error) => {
    const field = fieldNameForError(error);
    if (field && !result[field]) {
      result[field] = friendlyErrorMessage(error);
    }

    return result;
  }, {});
}

function fieldNameForError(error: ErrorObject): string | null {
  if (error.keyword === "required" && "missingProperty" in error.params) {
    return String(error.params.missingProperty);
  }

  return error.instancePath.replace(/^\//, "").split("/")[0] || null;
}

function friendlyErrorMessage(error: ErrorObject): string {
  if (error.keyword === "required") {
    return "Required";
  }

  if (error.keyword === "type" && "type" in error.params) {
    return `Must be a ${String(error.params.type)}`;
  }

  if (error.keyword === "enum") {
    return "Must match one of the allowed options";
  }

  if (error.keyword === "minimum" || error.keyword === "maximum") {
    return error.message ?? "Outside the allowed range";
  }

  if (error.keyword === "pattern") {
    return "Does not match the required pattern";
  }

  return error.message ?? "Invalid value";
}

function cleanObject<T extends Record<string, unknown>>(value: T): T {
  return Object.fromEntries(Object.entries(value).filter((entry) => entry[1] !== undefined)) as T;
}

function emptyToUndefined(value?: string | null): string | undefined {
  const normalized = value?.trim();
  return normalized ? normalized : undefined;
}

function emptyToNull(value?: string | null): string | null {
  return emptyToUndefined(value) ?? null;
}
