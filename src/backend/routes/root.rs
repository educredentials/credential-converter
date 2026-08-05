use axum::{
    response::Html,
    // routing::post,
    // Router,
};

pub async fn root() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head>
                <title>Open Badges to ELM converter</title>
                <style>
                    * {
                        box-sizing: border-box;
                    }
                    body {
                        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
                        max-width: 720px;
                        margin: 48px auto;
                        padding: 0 24px;
                        line-height: 1.6;
                        color: #1a1a1a;
                        background: #fafafa;
                    }
                    h1 {
                        font-size: 1.85rem;
                        margin-bottom: 16px;
                    }
                    p {
                        margin-bottom: 12px;
                    }
                    form {
                        background: #fff;
                        border: 1px solid #ddd;
                        border-radius: 8px;
                        padding: 24px;
                        margin: 24px 0;
                        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
                    }
                    label {
                        display: block;
                        font-weight: 600;
                        margin-bottom: 6px;
                    }
                    input[type="file"],
                    select {
                        display: block;
                        width: 100%;
                        margin-bottom: 16px;
                        padding: 8px;
                        border: 1px solid #ccc;
                        border-radius: 4px;
                        font-size: 0.95rem;
                    }
                    input[type="submit"] {
                        background: #772583;
                        color: #fff;
                        border: none;
                        border-radius: 4px;
                        padding: 10px 20px;
                        font-size: 0.95rem;
                        cursor: pointer;
                    }
                    input[type="submit"]:hover {
                        background: #5e1d68;
                    }
                    a {
                        color: #772583;
                    }
                </style>
            </head>
            <body>
                <h1>Open Badges to ELM converter</h1>
                <p>
                    This from is part of a service to translate Open Badges to ELM European Digital Credentials.</br>
                    The code as two parts a webservice that is called through this form and a CLI.</br>
                    The CLI can be found at the <a href="http://github.com/educredentials/">Educredentials github respository</a> 
                </p>
                <form action="/translate_file" method="post" enctype="multipart/form-data">
                    <label>
                        File to translate:
                        <input type="file" name="input_file" multiple>
                    </label>
                    <p>
                    <label>
                        type of translation:
                            <select id="translation" name="translation">
                            <option value="">--Please choose an option--</option>
                            <option value="OBv3ToELM">OBv3ToELM</option>
                            <option value="ELMToOBv3">ELMToOBv3</option>
                            </select>
                    </label>
                    <p>
                    <input type="submit" value="Translate this file">
                </form>
                <p>
                for testing of the created credential in ELM format check <a href="https://europa.eu/europass/digital-credentials/viewer/#/home">here</a>
                </p>
            </body>
        </html>
        "#,
    )
}
