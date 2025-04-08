# Latex

The API provides a LaTeX renderer to use, which is implemented using the [`tectonic`](https://crates.io/crates/tectonic) engine.

## `POST /latex/render`

### Request Body

The request body must have one of the following `Content-Type` values:

- `text/x-tex`
- `application/x-latex`
- `text/plain`

It should contain the LaTeX document to render.

### Response Body

If the engine succeeded in rendering the document, the response will be a PDF file with the `Content-Type` of `application/pdf`.

If the engine failed:

- The status code will be `400 Bad Request`
- The response will be the following extension of the default error object:

  | Field   | Type   | Description                                   |
  | ------- | ------ | --------------------------------------------- |
  | status  | number | The status code of the error                  |
  | message | string | The message of the error                      |
  | detail  | object | [Detailed LaTeX Error](#detailed-latex-error) |

## Data Structures

### Detailed LaTeX Error

| Field  | Type   | Description                                                           |
| ------ | ------ | --------------------------------------------------------------------- |
| error  | string | Detailed message of the error returned by the tectonic crate          |
| status | string | Status messages returned by the LaTeX engine (seperated by a newline) |
